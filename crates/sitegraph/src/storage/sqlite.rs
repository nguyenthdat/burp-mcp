use crate::graph::neighbors::{Neighbor, NeighborPage};
use crate::graph::traversal::{TracePage, TraceStep};
use crate::ingest::sitemap::relationships;
use crate::limits::{PageLimit, TraversalDepth};
use crate::model::{
    Endpoint, EndpointPage, GraphStatus, NodeKind, SyncBatch, SyncContext, SyncCoverage, SyncSummary,
};
use crate::normalize::{fingerprint, headers, url};
use crate::storage::{StorageError, edges, migrations::MIGRATOR, nodes, query::validated_limit};
use serde_json::json;
use sqlx::{ConnectOptions, Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;
use std::str::FromStr;
use time::OffsetDateTime;

pub struct SiteGraph {
    pool: SqlitePool,
    graph_id: String,
}

async fn upsert_source_node(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node: &crate::model::Node,
    search: nodes::SearchFields<'_>,
    context: &SyncContext,
    run_id: &str,
    timestamp: i64,
) -> Result<(), StorageError> {
    nodes::upsert(transaction, node, search).await?;
    sqlx::query(
        "INSERT INTO source_nodes(graph_id, source, scope, node_id, last_seen_run_id, last_seen_at, active)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, 1)
         ON CONFLICT(graph_id, source, scope, node_id) DO UPDATE SET
           last_seen_run_id=excluded.last_seen_run_id,
           last_seen_at=excluded.last_seen_at,
           active=1",
    )
    .bind(&context.graph_id)
    .bind(&context.source)
    .bind(&context.scope)
    .bind(&node.id)
    .bind(run_id)
    .bind(timestamp)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "DELETE FROM tombstones
         WHERE graph_id=?1 AND entity_type='node' AND entity_id=?2 AND source=?3 AND scope=?4",
    )
    .bind(&context.graph_id)
    .bind(&node.id)
    .bind(&context.source)
    .bind(&context.scope)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn upsert_source_edge(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    from_id: &str,
    to_id: &str,
    kind: crate::model::EdgeKind,
    evidence_id: &str,
    context: &SyncContext,
    run_id: &str,
    timestamp: i64,
) -> Result<String, StorageError> {
    let edge_id = edges::upsert(
        transaction,
        from_id,
        to_id,
        kind,
        evidence_id,
        timestamp,
    )
    .await?;
    sqlx::query(
        "INSERT INTO source_edges(graph_id, source, scope, edge_id, last_seen_run_id, last_seen_at, active)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, 1)
         ON CONFLICT(graph_id, source, scope, edge_id) DO UPDATE SET
           last_seen_run_id=excluded.last_seen_run_id,
           last_seen_at=excluded.last_seen_at,
           active=1",
    )
    .bind(&context.graph_id)
    .bind(&context.source)
    .bind(&context.scope)
    .bind(&edge_id)
    .bind(run_id)
    .bind(timestamp)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "DELETE FROM tombstones
         WHERE graph_id=?1 AND entity_type='edge' AND entity_id=?2 AND source=?3 AND scope=?4",
    )
    .bind(&context.graph_id)
    .bind(&edge_id)
    .bind(&context.source)
    .bind(&context.scope)
    .execute(&mut **transaction)
    .await?;
    Ok(edge_id)
}

impl SiteGraph {
    pub async fn open(path: &Path) -> Result<Self, StorageError> {
        let graph_id = path
            .file_stem()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or("default")
            .to_owned();
        Self::open_with_id(path, graph_id).await
    }

    pub async fn open_with_id(
        path: &Path,
        graph_id: impl Into<String>,
    ) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.display()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true)
            .disable_statement_logging();
        let pool = SqlitePool::connect_with(options).await?;
        MIGRATOR.run(&pool).await?;
        let graph_id = graph_id.into();
        sqlx::query(
            "INSERT INTO graph_metadata(key, value) VALUES('graph_id', ?1)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        )
        .bind(&graph_id)
        .execute(&pool)
        .await?;
        Ok(Self { pool, graph_id })
    }

    #[cfg(test)]
    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn sync(&self, batch: &SyncBatch) -> Result<SyncSummary, StorageError> {
        let mut context = SyncContext::snapshot(&self.graph_id, "all");
        context.items_seen = u64::try_from(batch.sitemap.len() + batch.issues.len())
            .unwrap_or(u64::MAX);
        self.sync_with_context(batch, &context).await
    }

    pub async fn sync_with_context(
        &self,
        batch: &SyncBatch,
        context: &SyncContext,
    ) -> Result<SyncSummary, StorageError> {
        let started = OffsetDateTime::now_utc();
        let now = started.unix_timestamp();
        let sync_id = if context.run_id.is_empty() {
            fingerprint::stable_id(
                "sync_run",
                &[&self.graph_id, &started.unix_timestamp_nanos().to_string()],
            )
        } else {
            context.run_id.clone()
        };
        let evidence_id = fingerprint::stable_id(
            "evidence",
            &[&self.graph_id, &context.source, &context.scope],
        );
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO sync_runs(id, graph_id, source, scope, started_at, status, complete, items_seen, pages_seen)
             VALUES(?1, ?2, ?3, ?4, ?5, 'running', 0, 0, 0)
             ON CONFLICT(id) DO NOTHING",
        )
        .bind(&sync_id)
        .bind(&self.graph_id)
        .bind(&context.source)
        .bind(&context.scope)
        .bind(now)
        .execute(&mut *transaction)
        .await?;
        sqlx::query("INSERT OR IGNORE INTO evidence(id, source, observed_at, summary) VALUES(?1, ?2, ?3, ?4)")
            .bind(&evidence_id)
            .bind(&context.source)
            .bind(now)
            .bind(json!({"sitemap_items": batch.sitemap.len(), "issue_items": batch.issues.len()}).to_string())
            .execute(&mut *transaction)
            .await?;
        let mut upserted_nodes = 0_u64;
        let mut upserted_edges = 0_u64;
        for observation in &batch.sitemap {
            let normalized =
                url::normalize(&observation.url).map_err(StorageError::InvalidInput)?;
            let method = observation.method.to_ascii_uppercase();
            let endpoint_hash = fingerprint::stable_id(
                "endpoint",
                &[&normalized.origin, &method, &normalized.path],
            );
            let endpoint = nodes::node(
                NodeKind::Endpoint,
                endpoint_hash.clone(),
                now,
                json!({
                    "origin": normalized.origin,
                    "method": method,
                    "path": normalized.path,
                    "status": observation.status,
                    "content_type": headers::content_type(&observation.content_type),
                    "response_fingerprint": fingerprint::response(&observation.response_body),
                    "parameter_names": normalized.parameter_names,
                }),
            );
            upsert_source_node(&mut transaction, &endpoint, nodes::SearchFields {
                origin: endpoint.metadata["origin"].as_str().unwrap_or_default(),
                method: endpoint.metadata["method"].as_str().unwrap_or_default(),
                path: endpoint.metadata["path"].as_str().unwrap_or_default(),
                name: "",
            }, context, &sync_id, now).await?;
            upserted_nodes += 1;
            let origin = nodes::node(
                NodeKind::Origin,
                fingerprint::stable_id(
                    "origin",
                    &[endpoint.metadata["origin"].as_str().unwrap_or_default()],
                ),
                now,
                json!({"origin": endpoint.metadata["origin"]}),
            );
            if !observation.response_body.is_empty() {
                let body_hash = blake3::hash(&observation.response_body).to_hex().to_string();
                let blob_id = fingerprint::stable_id("evidence_blob", &["response", &body_hash]);
                sqlx::query(
                    "INSERT OR IGNORE INTO evidence_blobs(id, sha256, blake3, source_entry_id, surface, direction, content_type, payload, byte_length, observed_at)
                     VALUES(?1, ?2, ?2, ?3, 'response_body', 'response', ?4, ?5, ?6, ?7)",
                )
                .bind(&blob_id)
                .bind(&body_hash)
                .bind(&endpoint.id)
                .bind(&observation.content_type)
                .bind(&observation.response_body)
                .bind(i64::try_from(observation.response_body.len()).unwrap_or(i64::MAX))
                .bind(now)
                .execute(&mut *transaction)
                .await?;
                let body_text = String::from_utf8_lossy(&observation.response_body);
                let secret_pattern = regex::Regex::new(r#"(?i)(?:token|secret|api[_-]?key|authorization)\s*[:=]\s*([^\s&\"']+)"#)
                    .map_err(|error| StorageError::InvalidInput(error.to_string()))?;
                for capture in secret_pattern.captures_iter(&body_text).take(128) {
                    let Some(found) = capture.get(1) else { continue };
                    let capture_bytes = found.as_str().as_bytes();
                    let finding_id = fingerprint::stable_id(
                        "exact_finding",
                        &[&endpoint.id, &blob_id, "secret_pattern", &found.start().to_string(), &found.end().to_string()],
                    );
                    sqlx::query(
                        "INSERT OR IGNORE INTO enrichment_findings(id, node_id, evidence_blob_id, enricher_id, enricher_version, ruleset_id, ruleset_version, input_fingerprint, kind, severity, confidence, byte_start, byte_end, capture, incomplete, metadata, observed_at)
                         VALUES(?1, ?2, ?3, 'secret_pattern', '1', 'default', '1', ?4, 'secret_like_value', 'high', 0.8, ?5, ?6, ?7, 0, ?8, ?9)",
                    )
                    .bind(&finding_id)
                    .bind(&endpoint.id)
                    .bind(&blob_id)
                    .bind(&body_hash)
                    .bind(i64::try_from(found.start()).unwrap_or(i64::MAX))
                    .bind(i64::try_from(found.end()).unwrap_or(i64::MAX))
                    .bind(capture_bytes)
                    .bind(json!({"capture_policy": "exact", "source": "response_body"}).to_string())
                    .bind(now)
                    .execute(&mut *transaction)
                    .await?;
                }
            }
            upsert_source_node(&mut transaction, &origin, nodes::SearchFields {
                origin: endpoint.metadata["origin"].as_str().unwrap_or_default(),
                ..nodes::SearchFields::default()
            }, context, &sync_id, now).await?;
            upsert_source_edge(&mut transaction, &origin.id, &endpoint.id, crate::model::EdgeKind::Contains, &evidence_id, context, &sync_id, now).await?;
            upserted_nodes += 1;
            upserted_edges += 1;
            let mut parent_id = origin.id.clone();
            let mut accumulated = String::new();
            for segment in endpoint.metadata["path"]
                .as_str()
                .unwrap_or_default()
                .split('/')
                .filter(|segment| !segment.is_empty())
            {
                accumulated.push('/');
                accumulated.push_str(segment);
                let segment_node = nodes::node(
                    NodeKind::PathSegment,
                    fingerprint::stable_id(
                        "path_segment",
                        &[
                            endpoint.metadata["origin"].as_str().unwrap_or_default(),
                            &accumulated,
                        ],
                    ),
                    now,
                    json!({"segment": segment, "path": accumulated}),
                );
                upsert_source_node(&mut transaction, &segment_node, nodes::SearchFields {
                    origin: endpoint.metadata["origin"].as_str().unwrap_or_default(),
                    path: segment_node.metadata["path"].as_str().unwrap_or_default(),
                    ..nodes::SearchFields::default()
                }, context, &sync_id, now).await?;
                upsert_source_edge(&mut transaction, &parent_id, &segment_node.id, crate::model::EdgeKind::PathChild, &evidence_id, context, &sync_id, now).await?;
                parent_id = segment_node.id.clone();
                upserted_nodes += 1;
                upserted_edges += 1;
            }
            for parameter_name in endpoint.metadata["parameter_names"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str())
            {
                let parameter = nodes::node(
                    NodeKind::Parameter,
                    fingerprint::stable_id("parameter", &[&endpoint.id, "query", parameter_name]),
                    now,
                    json!({"name": parameter_name, "location": "query"}),
                );
                upsert_source_node(&mut transaction, &parameter, nodes::SearchFields {
                    name: parameter_name,
                    ..nodes::SearchFields::default()
                }, context, &sync_id, now).await?;
                upsert_source_edge(&mut transaction, &endpoint.id, &parameter.id, crate::model::EdgeKind::AcceptsParameter, &evidence_id, context, &sync_id, now).await?;
                upserted_nodes += 1;
                upserted_edges += 1;
            }
            if let Some(response_hash) = endpoint.metadata["response_fingerprint"].as_str() {
                let response = nodes::node(
                    NodeKind::ResponseFingerprint,
                    fingerprint::stable_id("response_fingerprint", &[response_hash]),
                    now,
                    json!({"fingerprint": response_hash, "content_type": endpoint.metadata["content_type"]}),
                );
                upsert_source_node(&mut transaction, &response, nodes::SearchFields::default(), context, &sync_id, now).await?;
                upsert_source_edge(&mut transaction, &endpoint.id, &response.id, crate::model::EdgeKind::RespondedWith, &evidence_id, context, &sync_id, now).await?;
                upserted_nodes += 1;
                upserted_edges += 1;
            }
            for relationship in relationships(observation) {
                let target =
                    url::normalize(&relationship.target_url).map_err(StorageError::InvalidInput)?;
                let target_hash =
                    fingerprint::stable_id("endpoint", &[&target.origin, "GET", &target.path]);
                let target_node = nodes::node(
                    NodeKind::Endpoint,
                    target_hash.clone(),
                    now,
                    json!({"origin": target.origin, "method": "GET", "path": target.path}),
                );
                upsert_source_node(&mut transaction, &target_node, nodes::SearchFields {
                    origin: target_node.metadata["origin"].as_str().unwrap_or_default(),
                    method: "GET",
                    path: target_node.metadata["path"].as_str().unwrap_or_default(),
                    name: "",
                }, context, &sync_id, now).await?;
                upserted_nodes += 1;
                let kind = match relationship.kind {
                    "form" => crate::model::EdgeKind::FormSubmitsTo,
                    "script" => crate::model::EdgeKind::LoadsScript,
                    "redirect" => crate::model::EdgeKind::RedirectsTo,
                    _ => crate::model::EdgeKind::LinksTo,
                };
                upsert_source_edge(&mut transaction, &endpoint.id, &target_node.id, kind, &evidence_id, context, &sync_id, now).await?;
                upserted_edges += 1;
            }
        }
        let synced_origins = batch
            .sitemap
            .iter()
            .filter_map(|observation| {
                url::normalize(&observation.url)
                    .ok()
                    .map(|normalized| normalized.origin)
            })
            .collect::<std::collections::HashSet<_>>();
        for issue in &batch.issues {
            let issue_origin = url::normalize(&issue.url)
                .map_err(StorageError::InvalidInput)?
                .origin;
            if !synced_origins.is_empty() && !synced_origins.contains(&issue_origin) {
                continue;
            }
            let normalized = url::normalize(&issue.url).map_err(StorageError::InvalidInput)?;
            let endpoint_hash =
                fingerprint::stable_id("endpoint", &[&normalized.origin, "GET", &normalized.path]);
            let endpoint = nodes::node(
                NodeKind::Endpoint,
                endpoint_hash,
                now,
                json!({"origin": normalized.origin, "method": "GET", "path": normalized.path}),
            );
            upsert_source_node(&mut transaction, &endpoint, nodes::SearchFields {
                origin: endpoint.metadata["origin"].as_str().unwrap_or_default(),
                method: "GET",
                path: endpoint.metadata["path"].as_str().unwrap_or_default(),
                name: "",
            }, context, &sync_id, now).await?;
            let issue_hash = fingerprint::stable_id(
                "issue",
                &[
                    &endpoint.id,
                    &issue.name,
                    &issue.severity,
                    &issue.confidence,
                ],
            );
            let issue_node = nodes::node(
                NodeKind::Issue,
                issue_hash,
                now,
                json!({
                    "name": issue.name,
                    "severity": issue.severity,
                    "confidence": issue.confidence,
                }),
            );
            upsert_source_node(&mut transaction, &issue_node, nodes::SearchFields {
                name: &issue.name,
                ..nodes::SearchFields::default()
            }, context, &sync_id, now).await?;
            upsert_source_edge(&mut transaction, &endpoint.id, &issue_node.id, crate::model::EdgeKind::HasIssue, &evidence_id, context, &sync_id, now).await?;
            upserted_nodes += 2;
            upserted_edges += 1;
        }
        for technology in &batch.technologies {
            let normalized =
                url::normalize(&technology.endpoint_url).map_err(StorageError::InvalidInput)?;
            let endpoint_id =
                fingerprint::stable_id("endpoint", &[&normalized.origin, "GET", &normalized.path]);
            let endpoint = nodes::node(
                NodeKind::Endpoint,
                endpoint_id,
                now,
                json!({"origin": normalized.origin, "method": "GET", "path": normalized.path}),
            );
            upsert_source_node(&mut transaction, &endpoint, nodes::SearchFields {
                origin: endpoint.metadata["origin"].as_str().unwrap_or_default(),
                method: "GET",
                path: endpoint.metadata["path"].as_str().unwrap_or_default(),
                name: "",
            }, context, &sync_id, now).await?;
            let normalized_name = technology.name.trim().to_ascii_lowercase();
            let technology_node = nodes::node(
                NodeKind::Technology,
                fingerprint::stable_id("technology", &[&normalized_name]),
                now,
                json!({"name": normalized_name}),
            );
            upsert_source_node(&mut transaction, &technology_node, nodes::SearchFields {
                name: technology_node.metadata["name"]
                    .as_str()
                    .unwrap_or_default(),
                ..nodes::SearchFields::default()
            }, context, &sync_id, now).await?;
            upsert_source_edge(&mut transaction, &endpoint.id, &technology_node.id, crate::model::EdgeKind::HasTechnology, &evidence_id, context, &sync_id, now).await?;
            upserted_nodes += 2;
            upserted_edges += 1;
        }
        for artifact in &batch.artifacts {
            let normalized =
                url::normalize(&artifact.endpoint_url).map_err(StorageError::InvalidInput)?;
            let endpoint_id =
                fingerprint::stable_id("endpoint", &[&normalized.origin, "GET", &normalized.path]);
            let endpoint = nodes::node(
                NodeKind::Endpoint,
                endpoint_id,
                now,
                json!({"origin": normalized.origin, "method": "GET", "path": normalized.path}),
            );
            upsert_source_node(&mut transaction, &endpoint, nodes::SearchFields {
                origin: endpoint.metadata["origin"].as_str().unwrap_or_default(),
                method: "GET",
                path: endpoint.metadata["path"].as_str().unwrap_or_default(),
                name: "",
            }, context, &sync_id, now).await?;
            let kind = artifact.kind.trim().to_ascii_lowercase();
            let name = artifact.name.trim();
            let artifact_node = nodes::node(
                NodeKind::Artifact,
                fingerprint::stable_id("artifact", &[&kind, name, &artifact.fingerprint]),
                now,
                json!({"kind": kind, "name": name, "fingerprint": artifact.fingerprint}),
            );
            upsert_source_node(&mut transaction, &artifact_node, nodes::SearchFields {
                name: artifact_node.metadata["name"].as_str().unwrap_or_default(),
                ..nodes::SearchFields::default()
            }, context, &sync_id, now).await?;
            upsert_source_edge(&mut transaction, &endpoint.id, &artifact_node.id, crate::model::EdgeKind::HasArtifact, &evidence_id, context, &sync_id, now).await?;
            upserted_nodes += 2;
            upserted_edges += 1;
        }
        let mut tombstoned_nodes = 0_u64;
        let mut tombstoned_edges = 0_u64;
        if context.complete {
            tombstoned_nodes = sqlx::query(
                "SELECT count(*) AS count FROM source_nodes
                 WHERE graph_id=?1 AND source=?2 AND scope=?3 AND active=1 AND last_seen_run_id<>?4",
            )
            .bind(&self.graph_id)
            .bind(&context.source)
            .bind(&context.scope)
            .bind(&sync_id)
            .fetch_one(&mut *transaction)
            .await?
            .get::<i64, _>("count") as u64;
            tombstoned_edges = sqlx::query(
                "SELECT count(*) AS count FROM source_edges
                 WHERE graph_id=?1 AND source=?2 AND scope=?3 AND active=1 AND last_seen_run_id<>?4",
            )
            .bind(&self.graph_id)
            .bind(&context.source)
            .bind(&context.scope)
            .bind(&sync_id)
            .fetch_one(&mut *transaction)
            .await?
            .get::<i64, _>("count") as u64;
            sqlx::query(
                "INSERT INTO tombstones(graph_id, entity_type, entity_id, source, scope, first_missing_at, last_confirmed_at, reason)
                 SELECT graph_id, 'node', node_id, source, scope, ?5, ?5, 'missing_from_complete_snapshot'
                 FROM source_nodes
                 WHERE graph_id=?1 AND source=?2 AND scope=?3 AND active=1 AND last_seen_run_id<>?4
                 ON CONFLICT(graph_id, entity_type, entity_id, source, scope)
                 DO UPDATE SET last_confirmed_at=excluded.last_confirmed_at",
            )
            .bind(&self.graph_id)
            .bind(&context.source)
            .bind(&context.scope)
            .bind(&sync_id)
            .bind(now)
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "INSERT INTO tombstones(graph_id, entity_type, entity_id, source, scope, first_missing_at, last_confirmed_at, reason)
                 SELECT graph_id, 'edge', edge_id, source, scope, ?5, ?5, 'missing_from_complete_snapshot'
                 FROM source_edges
                 WHERE graph_id=?1 AND source=?2 AND scope=?3 AND active=1 AND last_seen_run_id<>?4
                 ON CONFLICT(graph_id, entity_type, entity_id, source, scope)
                 DO UPDATE SET last_confirmed_at=excluded.last_confirmed_at",
            )
            .bind(&self.graph_id)
            .bind(&context.source)
            .bind(&context.scope)
            .bind(&sync_id)
            .bind(now)
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "UPDATE source_nodes SET active=0
                 WHERE graph_id=?1 AND source=?2 AND scope=?3 AND active=1 AND last_seen_run_id<>?4",
            )
            .bind(&self.graph_id)
            .bind(&context.source)
            .bind(&context.scope)
            .bind(&sync_id)
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "UPDATE source_edges SET active=0
                 WHERE graph_id=?1 AND source=?2 AND scope=?3 AND active=1 AND last_seen_run_id<>?4",
            )
            .bind(&self.graph_id)
            .bind(&context.source)
            .bind(&context.scope)
            .bind(&sync_id)
            .execute(&mut *transaction)
            .await?;
        }

        let coverage = SyncCoverage {
            complete: context.complete,
            items_indexed: context.items_seen,
            source_total: context.source_total,
            pages_read: context.pages_seen,
            end_of_source: context.complete,
            cancelled: false,
            last_cursor: context.cursor.clone(),
        };
        sqlx::query(
            "UPDATE sync_runs SET finished_at=?2, status=?3, complete=?4, items_seen=?5, pages_seen=?6, error=NULL
             WHERE id=?1",
        )
        .bind(&sync_id)
        .bind(now)
        .bind(if context.complete { "completed" } else { "running" })
        .bind(context.complete)
        .bind(i64::try_from(context.items_seen).unwrap_or(i64::MAX))
        .bind(i64::try_from(context.pages_seen).unwrap_or(i64::MAX))
        .execute(&mut *transaction)
        .await?;
        sqlx::query(
            "INSERT INTO source_checkpoints(graph_id, source, scope, last_cursor, last_snapshot_id, last_success_at, coverage_json)
             VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(graph_id, source, scope) DO UPDATE SET
               last_cursor=excluded.last_cursor,
               last_snapshot_id=excluded.last_snapshot_id,
               last_success_at=CASE WHEN excluded.last_success_at IS NULL THEN source_checkpoints.last_success_at ELSE excluded.last_success_at END,
               coverage_json=excluded.coverage_json",
        )
        .bind(&self.graph_id)
        .bind(&context.source)
        .bind(&context.scope)
        .bind(&context.cursor)
        .bind(&sync_id)
        .bind(context.complete.then_some(now))
        .bind(serde_json::to_string(&coverage)?)
        .execute(&mut *transaction)
        .await?;
        sqlx::query("INSERT INTO graph_metadata(key, value) VALUES('last_synced_at', ?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
            .bind(now.to_string())
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        let status = self.status().await?;
        Ok(SyncSummary {
            sync_id,
            upserted_nodes,
            upserted_edges,
            total_nodes: status.total_nodes,
            total_edges: status.total_edges,
            last_synced_at: now,
            complete: context.complete,
            items_seen: context.items_seen,
            pages_seen: context.pages_seen,
            tombstoned_nodes,
            tombstoned_edges,
        })
    }

    pub async fn status(&self) -> Result<GraphStatus, StorageError> {
        let node_count = sqlx::query("SELECT count(*) AS count FROM nodes")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count");
        let edge_count = sqlx::query("SELECT count(*) AS count FROM edges")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count");
        let active_nodes = sqlx::query(
            "SELECT count(*) AS count FROM source_nodes WHERE graph_id=?1 AND active=1",
        )
        .bind(&self.graph_id)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>("count");
        let active_edges = sqlx::query(
            "SELECT count(*) AS count FROM source_edges WHERE graph_id=?1 AND active=1",
        )
        .bind(&self.graph_id)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>("count");
        let last_synced_at = sqlx::query(
            "SELECT value FROM graph_metadata WHERE key='last_synced_at'",
        )
        .fetch_optional(&self.pool)
        .await?
        .and_then(|row| row.get::<String, _>("value").parse().ok());
        let checkpoint = sqlx::query(
            "SELECT last_success_at, last_snapshot_id, coverage_json
             FROM source_checkpoints WHERE graph_id=?1
             ORDER BY last_success_at DESC LIMIT 1",
        )
        .bind(&self.graph_id)
        .fetch_optional(&self.pool)
        .await?;
        let (last_success_at, current_run_id, coverage) = match checkpoint {
            Some(row) => (
                row.get::<Option<i64>, _>("last_success_at"),
                row.get::<Option<String>, _>("last_snapshot_id"),
                serde_json::from_str::<SyncCoverage>(&row.get::<String, _>("coverage_json"))?,
            ),
            None => (None, None, SyncCoverage::default()),
        };
        let last_error = sqlx::query(
            "SELECT error FROM sync_runs
             WHERE graph_id=?1 AND error IS NOT NULL
             ORDER BY started_at DESC LIMIT 1",
        )
        .bind(&self.graph_id)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| row.get::<String, _>("error"));
        Ok(GraphStatus {
            graph_id: self.graph_id.clone(),
            schema_version: 3,
            state: if last_synced_at.is_none() {
                "disabled"
            } else if coverage.complete {
                "ready"
            } else {
                "catching_up"
            }
            .to_owned(),
            freshness: if coverage.complete { "fresh" } else { "partial" }.to_owned(),
            total_nodes: node_count as u64,
            total_edges: edge_count as u64,
            active_nodes: active_nodes as u64,
            active_edges: active_edges as u64,
            last_synced_at,
            last_success_at,
            current_run_id,
            coverage,
            last_error,
        })
    }

    pub async fn search(
        &self,
        query: &str,
        cursor: u64,
        limit: u64,
    ) -> Result<EndpointPage, StorageError> {
        let limit = validated_limit(limit)?;
        let pattern = literal_prefix_pattern(query);
        if pattern.is_empty() {
            return Ok(EndpointPage {
                items: Vec::new(),
                total: 0,
                truncated: false,
                next_cursor: None,
                last_synced_at: self.status().await?.last_synced_at,
                evidence: json!({}),
            });
        }
        let total = sqlx::query("SELECT count(*) AS count FROM node_search JOIN nodes n ON n.id=node_search.node_id WHERE node_search MATCH ?1 AND n.kind='endpoint'")
            .bind(&pattern)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as u64;
        let rows = sqlx::query("SELECT n.id, n.metadata, n.updated_at FROM node_search JOIN nodes n ON n.id=node_search.node_id WHERE node_search MATCH ?1 AND n.kind='endpoint' ORDER BY n.id LIMIT ?2 OFFSET ?3")
            .bind(&pattern).bind(limit as i64).bind(cursor as i64).fetch_all(&self.pool).await?;
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let metadata: serde_json::Value =
                serde_json::from_str(&row.get::<String, _>("metadata"))?;
            items.push(Endpoint {
                id: row.get("id"),
                origin: metadata["origin"].as_str().unwrap_or_default().to_owned(),
                method: metadata["method"].as_str().unwrap_or_default().to_owned(),
                path: metadata["path"].as_str().unwrap_or_default().to_owned(),
                status: metadata["status"].as_u64().unwrap_or_default() as u32,
                content_type: metadata["content_type"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                response_fingerprint: metadata["response_fingerprint"].as_str().map(str::to_owned),
                parameter_names: metadata["parameter_names"]
                    .as_array()
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default(),
                last_seen_at: row.get("updated_at"),
            });
        }
        let next = cursor + items.len() as u64;
        Ok(EndpointPage {
            items,
            total,
            truncated: next < total,
            next_cursor: (next < total).then_some(next),
            last_synced_at: self.status().await?.last_synced_at,
            evidence: json!({"source": "SQLite FTS5 node metadata"}),
        })
    }

    pub async fn endpoint(&self, id: &str) -> Result<Option<Endpoint>, StorageError> {
        let Some(row) = sqlx::query(
            "SELECT id, metadata, updated_at FROM nodes WHERE id=?1 AND kind='endpoint'",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };
        let metadata: serde_json::Value = serde_json::from_str(&row.get::<String, _>("metadata"))?;
        Ok(Some(Endpoint {
            id: row.get("id"),
            origin: metadata["origin"].as_str().unwrap_or_default().to_owned(),
            method: metadata["method"].as_str().unwrap_or_default().to_owned(),
            path: metadata["path"].as_str().unwrap_or_default().to_owned(),
            status: metadata["status"].as_u64().unwrap_or_default() as u32,
            content_type: metadata["content_type"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            response_fingerprint: metadata["response_fingerprint"].as_str().map(str::to_owned),
            parameter_names: metadata["parameter_names"]
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            last_seen_at: row.get("updated_at"),
        }))
    }

    pub async fn neighbors(
        &self,
        node_id: &str,
        cursor: u64,
        limit: u64,
    ) -> Result<NeighborPage, StorageError> {
        let limit = validated_limit(limit)?;
        let total = sqlx::query("SELECT count(*) AS count FROM edges WHERE from_id=?1 OR to_id=?1")
            .bind(node_id)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as u64;
        let rows = sqlx::query("SELECT e.id AS edge_id, e.kind, e.from_id, e.to_id, n.id AS node_id, n.kind AS node_kind, n.metadata FROM edges e JOIN nodes n ON n.id=CASE WHEN e.from_id=?1 THEN e.to_id ELSE e.from_id END WHERE e.from_id=?1 OR e.to_id=?1 ORDER BY e.id LIMIT ?2 OFFSET ?3")
            .bind(node_id).bind(limit as i64).bind(cursor as i64).fetch_all(&self.pool).await?;
        let items = rows
            .into_iter()
            .map(|row| Neighbor {
                edge_id: row.get("edge_id"),
                kind: row.get("kind"),
                direction: if row.get::<String, _>("from_id") == node_id {
                    "outgoing".to_owned()
                } else {
                    "incoming".to_owned()
                },
                node_id: row.get("node_id"),
                node_kind: row.get("node_kind"),
                metadata: serde_json::from_str(&row.get::<String, _>("metadata"))
                    .unwrap_or_else(|_| json!({})),
            })
            .collect::<Vec<_>>();
        let next = cursor + items.len() as u64;
        Ok(NeighborPage {
            items,
            total,
            truncated: next < total,
            next_cursor: (next < total).then_some(next),
            last_synced_at: self.status().await?.last_synced_at,
            evidence: json!({"source": "SQLite adjacency edges"}),
        })
    }

    pub async fn diff(
        &self,
        since: i64,
        cursor: u64,
        limit: u64,
    ) -> Result<crate::graph::diff::GraphDiff, StorageError> {
        let limit = validated_limit(limit)?;
        crate::graph::diff::since(
            &self.pool,
            since,
            cursor,
            limit,
            self.status().await?.last_synced_at,
        )
        .await
    }

    pub async fn export_json(
        &self,
        cursor: u64,
        limit: u64,
    ) -> Result<crate::export::json::JsonExport, StorageError> {
        let limit = validated_limit(limit)?;
        crate::export::json::page(
            &self.pool,
            cursor,
            limit,
            self.status().await?.last_synced_at,
        )
        .await
    }

    pub async fn export_csv(
        &self,
        cursor: u64,
        limit: u64,
    ) -> Result<crate::export::csv::CsvExport, StorageError> {
        let limit = validated_limit(limit)?;
        crate::export::csv::page(
            &self.pool,
            cursor,
            limit,
            self.status().await?.last_synced_at,
        )
        .await
    }

    pub async fn trace(
        &self,
        start_id: &str,
        max_depth: u32,
        limit: u32,
    ) -> Result<TracePage, StorageError> {
        let depth = TraversalDepth::new(max_depth)?.get();
        let limit = PageLimit::new(limit)?.get();
        let rows = sqlx::query("WITH RECURSIVE walk(depth, edge_id, edge_kind, from_id, to_id, path, visited) AS (SELECT 1, e.id, e.kind, e.from_id, e.to_id, printf('%s>%s', e.from_id, e.to_id), printf('|%s|%s|', e.from_id, e.to_id) FROM edges e WHERE e.from_id=?1 UNION ALL SELECT walk.depth+1, e.id, e.kind, e.from_id, e.to_id, printf('%s>%s', walk.path, e.to_id), walk.visited || e.to_id || '|' FROM walk JOIN edges e ON e.from_id=walk.to_id WHERE walk.depth < ?2 AND instr(walk.visited, printf('|%s|', e.to_id)) = 0) SELECT depth, edge_id, edge_kind, from_id, to_id, path FROM walk ORDER BY depth, edge_id LIMIT ?3")
            .bind(start_id)
            .bind(depth as i64)
            .bind(i64::from(limit) + 1)
            .fetch_all(&self.pool)
            .await?;
        let mut items = rows
            .into_iter()
            .map(|row| TraceStep {
                depth: row.get::<i64, _>("depth") as u32,
                edge_id: row.get("edge_id"),
                edge_kind: row.get("edge_kind"),
                from_id: row.get("from_id"),
                to_id: row.get("to_id"),
                path: row.get("path"),
            })
            .collect::<Vec<_>>();
        let truncated = items.len() > limit as usize;
        items.truncate(limit as usize);
        Ok(TracePage {
            total: items.len() as u64 + u64::from(truncated),
            truncated,
            next_cursor: None,
            last_synced_at: self.status().await?.last_synced_at,
            items,
        })
    }
}

fn literal_prefix_pattern(query: &str) -> String {
    query
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::MAX_TRAVERSAL_DEPTH;
    use crate::model::SitemapObservation;

    async fn graph() -> SiteGraph {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        MIGRATOR.run(&pool).await.unwrap();
        SiteGraph { pool, graph_id: "test".to_owned() }
    }

    fn observation(path: &str, body: &[u8]) -> SitemapObservation {
        SitemapObservation {
            url: format!("https://example.test/{path}?token=secret"),
            method: "GET".to_owned(),
            status: 200,
            content_type: "text/html".to_owned(),
            response_body: body.to_vec(),
            redirect_url: String::new(),
            response_links: Vec::new(),
            form_actions: Vec::new(),
            script_sources: Vec::new(),
        }
    }

    #[tokio::test]
    async fn repeated_sync_is_idempotent_and_does_not_store_bodies_or_values() {
        let graph = graph().await;
        let batch = SyncBatch {
            sitemap: vec![observation(
                "start",
                br#"<a href='/next?code=private'>next</a>"#,
            )],
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let first = graph.status().await.unwrap();
        graph.sync(&batch).await.unwrap();
        let second = graph.status().await.unwrap();
        assert_eq!(first.total_nodes, second.total_nodes);
        assert_eq!(first.total_edges, second.total_edges);
        let stored = sqlx::query("SELECT group_concat(metadata, '') AS value FROM nodes")
            .fetch_one(graph.pool())
            .await
            .unwrap()
            .get::<String, _>("value");
        assert!(!stored.contains("secret"));
        assert!(!stored.contains("private"));
        assert!(!stored.contains("<a"));
    }

    #[tokio::test]
    async fn traversal_enforces_depth_and_result_limits() {
        let graph = graph().await;
        let batch = SyncBatch {
            sitemap: (0..12)
                .map(|index| {
                    observation(
                        &format!("n{index}"),
                        format!("<a href='/n{}'>next</a>", index + 1).as_bytes(),
                    )
                })
                .collect(),
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let start = fingerprint::stable_id("endpoint", &["https://example.test", "GET", "/n0"]);
        let invalid_depth = graph.trace(&start, 100, 3).await.unwrap_err();
        assert!(invalid_depth.to_string().contains("max_depth"));
        let trace = graph.trace(&start, MAX_TRAVERSAL_DEPTH, 3).await.unwrap();
        assert_eq!(trace.items.len(), 3);
        assert!(
            trace
                .items
                .iter()
                .all(|item| item.depth <= MAX_TRAVERSAL_DEPTH)
        );
        assert!(trace.truncated);
    }

    #[tokio::test]
    async fn fts_pagination_is_deterministic_without_duplicates() {
        let graph = graph().await;
        let batch = SyncBatch {
            sitemap: (0..4)
                .map(|index| observation(&format!("search-{index}"), b""))
                .collect(),
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let first = graph.search("search", 0, 2).await.unwrap();
        let second = graph
            .search("search", first.next_cursor.unwrap(), 2)
            .await
            .unwrap();
        assert_eq!(first.items.len(), 2);
        assert_eq!(second.items.len(), 2);
        assert!(
            first
                .items
                .iter()
                .all(|left| second.items.iter().all(|right| left.id != right.id))
        );
    }

    #[tokio::test]
    async fn fts_literal_queries_are_safe_and_endpoint_only() {
        let graph = graph().await;
        graph
            .sync(&SyncBatch {
                sitemap: vec![observation("products/search", b"{}")],
                ..SyncBatch::default()
            })
            .await
            .unwrap();

        for query in [
            "products/search",
            "products-search",
            "https://example.test:443",
            "products\"search",
            "products*",
            "検索",
        ] {
            let page = graph.search(query, 0, 20).await.unwrap();
            assert!(page.items.iter().all(|item| !item.origin.is_empty()));
            assert!(page.items.iter().all(|item| !item.method.is_empty()));
        }

        assert!(graph.search("   ", 0, 20).await.unwrap().items.is_empty());
        let matching = graph.search("products/search", 0, 20).await.unwrap();
        assert_eq!(1, matching.items.len());
        assert_eq!("/products/search", matching.items[0].path);
    }

    #[tokio::test]
    async fn prefix_scoped_sync_excludes_unrelated_issues() {
        let graph = graph().await;
        graph
            .sync(&SyncBatch {
                sitemap: vec![observation("products/search", b"{}")],
                issues: vec![
                    crate::model::IssueObservation {
                        name: "local".to_owned(),
                        severity: "low".to_owned(),
                        confidence: "firm".to_owned(),
                        url: "https://example.test/products/search".to_owned(),
                    },
                    crate::model::IssueObservation {
                        name: "external".to_owned(),
                        severity: "low".to_owned(),
                        confidence: "firm".to_owned(),
                        url: "https://external.test/".to_owned(),
                    },
                ],
                ..SyncBatch::default()
            })
            .await
            .unwrap();

        let issue_names = sqlx::query(
            "SELECT metadata ->> '$.name' AS name FROM nodes WHERE kind='issue' ORDER BY name",
        )
        .fetch_all(graph.pool())
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<Vec<_>>();
        assert_eq!(vec!["local"], issue_names);
    }

    #[tokio::test]
    async fn issues_link_to_endpoints_without_storing_detail_values() {
        let graph = graph().await;
        let batch = SyncBatch {
            issues: vec![crate::model::IssueObservation {
                name: "Synthetic finding".to_owned(),
                severity: "Information".to_owned(),
                confidence: "Certain".to_owned(),
                url: "https://example.test/issue?secret=value".to_owned(),
            }],
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let status = graph.status().await.unwrap();
        assert_eq!(status.total_nodes, 2);
        assert_eq!(status.total_edges, 1);
        let stored = sqlx::query("SELECT group_concat(metadata, '') AS value FROM nodes")
            .fetch_one(graph.pool())
            .await
            .unwrap()
            .get::<String, _>("value");
        assert!(!stored.contains("secret"));
        assert!(!stored.contains("value"));
    }

    #[tokio::test]
    async fn exports_are_bounded_and_metadata_only() {
        let graph = graph().await;
        let batch = SyncBatch {
            sitemap: (0..3)
                .map(|index| observation(&format!("export-{index}"), b"private body"))
                .collect(),
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let export = graph.export_json(0, 2).await.unwrap();
        assert_eq!(export.nodes.len(), 2);
        assert!(export.truncated);
        assert!(
            !serde_json::to_string(&export)
                .unwrap()
                .contains("private body")
        );
        let csv = graph.export_csv(0, 2).await.unwrap();
        assert!(csv.truncated);
        assert!(!csv.csv.contains("private body"));
    }

    #[tokio::test]
    async fn file_database_reopens_and_recovers_interrupted_user_transaction() {
        let path = std::env::temp_dir().join(format!(
            "burp-mcp-sitegraph-{}-{}.sqlite",
            std::process::id(),
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        ));
        let graph = SiteGraph::open(&path).await.unwrap();
        graph
            .sync(&SyncBatch {
                sitemap: vec![observation("persistent", b"")],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        let mut transaction = graph.pool().begin().await.unwrap();
        sqlx::query("INSERT INTO graph_metadata(key, value) VALUES('interrupted', 'private')")
            .execute(&mut *transaction)
            .await
            .unwrap();
        transaction.rollback().await.unwrap();
        graph.pool().close().await;

        let reopened = SiteGraph::open(&path).await.unwrap();
        assert_eq!(reopened.status().await.unwrap().schema_version, 3);
        assert!(reopened.search("persistent", 0, 10).await.unwrap().total > 0);
        let interrupted = sqlx::query("SELECT value FROM graph_metadata WHERE key='interrupted'")
            .fetch_optional(reopened.pool())
            .await
            .unwrap();
        assert!(interrupted.is_none());
        reopened.pool().close().await;
        let _ = tokio::fs::remove_file(&path).await;
        let _ = tokio::fs::remove_file(path.with_extension("sqlite-wal")).await;
        let _ = tokio::fs::remove_file(path.with_extension("sqlite-shm")).await;
    }

    #[tokio::test]
    async fn technologies_and_artifacts_persist_as_searchable_linked_nodes() {
        let graph = graph().await;
        graph
            .sync(&SyncBatch {
                technologies: vec![crate::model::TechnologyObservation {
                    name: "Synthetic Runtime".to_owned(),
                    endpoint_url: "https://example.test/app?token=private".to_owned(),
                }],
                artifacts: vec![crate::model::ArtifactObservation {
                    kind: "schema".to_owned(),
                    name: "public-contract".to_owned(),
                    endpoint_url: "https://example.test/app?key=secret".to_owned(),
                    fingerprint: "sha256:synthetic".to_owned(),
                }],
                ..SyncBatch::default()
            })
            .await
            .unwrap();

        let technology = sqlx::query("SELECT id, metadata FROM nodes WHERE kind='technology'")
            .fetch_one(graph.pool())
            .await
            .unwrap();
        let artifact = sqlx::query("SELECT id, metadata FROM nodes WHERE kind='artifact'")
            .fetch_one(graph.pool())
            .await
            .unwrap();
        assert!(
            technology
                .get::<String, _>("metadata")
                .contains("synthetic runtime")
        );
        assert!(
            artifact
                .get::<String, _>("metadata")
                .contains("public-contract")
        );
        let linked = sqlx::query(
            "SELECT count(*) AS count FROM edges WHERE kind IN ('has_technology', 'has_artifact')",
        )
        .fetch_one(graph.pool())
        .await
        .unwrap()
        .get::<i64, _>("count");
        assert_eq!(linked, 2);
        let stored = sqlx::query("SELECT group_concat(metadata, '') AS value FROM nodes")
            .fetch_one(graph.pool())
            .await
            .unwrap()
            .get::<String, _>("value");
        assert!(!stored.contains("private"));
        assert!(!stored.contains("secret"));
    }


    #[tokio::test]
    async fn complete_snapshot_tombstones_missing_nodes_but_partial_snapshot_does_not() {
        let graph = graph().await;
        let mut first = SyncContext::snapshot("test", "all");
        first.run_id = "first".to_owned();
        first.items_seen = 2;
        graph
            .sync_with_context(
                &SyncBatch {
                    sitemap: vec![observation("kept", b""), observation("removed", b"")],
                    ..SyncBatch::default()
                },
                &first,
            )
            .await
            .unwrap();

        let mut partial = SyncContext::snapshot("test", "all");
        partial.run_id = "partial".to_owned();
        partial.items_seen = 1;
        partial.complete = false;
        let partial_summary = graph
            .sync_with_context(
                &SyncBatch {
                    sitemap: vec![observation("kept", b"")],
                    ..SyncBatch::default()
                },
                &partial,
            )
            .await
            .unwrap();
        assert_eq!(partial_summary.tombstoned_nodes, 0);

        let mut complete = SyncContext::snapshot("test", "all");
        complete.run_id = "complete".to_owned();
        complete.items_seen = 1;
        let complete_summary = graph
            .sync_with_context(
                &SyncBatch {
                    sitemap: vec![observation("kept", b"")],
                    ..SyncBatch::default()
                },
                &complete,
            )
            .await
            .unwrap();
        assert!(complete_summary.tombstoned_nodes > 0);
        let status = graph.status().await.unwrap();
        assert!(status.coverage.complete);
        assert_eq!(status.coverage.items_indexed, 1);
    }

    #[tokio::test]
    async fn repeated_sync_keeps_edge_and_evidence_identity_stable() {
        let graph = graph().await;
        let batch = SyncBatch {
            sitemap: vec![observation("stable", b"")],
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let first_edge_ids = sqlx::query("SELECT id FROM edges ORDER BY id")
            .fetch_all(graph.pool())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("id"))
            .collect::<Vec<_>>();
        let first_evidence_ids = sqlx::query("SELECT id FROM evidence ORDER BY id")
            .fetch_all(graph.pool())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("id"))
            .collect::<Vec<_>>();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        graph.sync(&batch).await.unwrap();
        let second_edge_ids = sqlx::query("SELECT id FROM edges ORDER BY id")
            .fetch_all(graph.pool())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("id"))
            .collect::<Vec<_>>();
        let second_evidence_ids = sqlx::query("SELECT id FROM evidence ORDER BY id")
            .fetch_all(graph.pool())
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>("id"))
            .collect::<Vec<_>>();
        assert_eq!(first_edge_ids, second_edge_ids);
        assert_eq!(first_evidence_ids, second_evidence_ids);
    }

    #[tokio::test]
    async fn exact_response_evidence_round_trips_byte_for_byte() {
        let graph = graph().await;
        let body = b"token=secret-cookie-value\0\xff";
        graph
            .sync(&SyncBatch {
                sitemap: vec![observation("evidence", body)],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        let stored = sqlx::query("SELECT payload, byte_length FROM evidence_blobs")
            .fetch_one(graph.pool())
            .await
            .unwrap();
        assert_eq!(stored.get::<Vec<u8>, _>("payload"), body);
        assert_eq!(stored.get::<i64, _>("byte_length"), body.len() as i64);
    }

}
