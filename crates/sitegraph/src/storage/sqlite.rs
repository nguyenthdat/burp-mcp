use crate::analysis;
use crate::enrichment::RulePack;
use crate::graph::neighbors::{Neighbor, NeighborPage};
use crate::graph::traversal::{TracePage, TraceStep};
use crate::ingest::relationships;
use crate::limits::{PageLimit, TraversalDepth};
use crate::model::{
    Endpoint, EndpointPage, EvidenceSource, GraphStatus, NodeKind, NodeMetadata, SyncBatch,
    SyncContext, SyncCoverage, SyncSummary,
};
use crate::normalize::{fingerprint, headers, url};
use crate::storage::evidence::{persist_rule_findings, upsert_evidence_blob};
use crate::storage::{StorageError, edges, migrations::MIGRATOR, nodes, query::validated_limit};
use sqlx::{ConnectOptions, Row, SqlitePool, sqlite::SqliteConnectOptions};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use time::OffsetDateTime;

pub struct SiteGraph {
    pool: SqlitePool,
    graph_id: String,
    rule_pack: Arc<RulePack>,
}

#[derive(serde::Serialize)]
struct SyncEvidenceSummary {
    sitemap_items: usize,
    issue_items: usize,
}

fn endpoint_from_metadata(id: String, last_seen_at: i64, metadata: NodeMetadata) -> Endpoint {
    Endpoint {
        id,
        origin: metadata.origin,
        method: metadata.method,
        path: metadata.path,
        status: metadata.status.unwrap_or_default(),
        content_type: metadata.content_type,
        response_fingerprint: metadata.response_fingerprint,
        parameter_names: metadata.parameter_names,
        last_seen_at,
    }
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

#[allow(clippy::too_many_arguments)]
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
    let edge_id = edges::upsert(transaction, from_id, to_id, kind, evidence_id, timestamp).await?;
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
        let rule_pack = RulePack::default_exact().map_err(StorageError::InvalidInput)?;
        Self::open_with_rules(path, graph_id, rule_pack).await
    }

    pub async fn open_with_rules(
        path: &Path,
        graph_id: impl Into<String>,
        rule_pack: RulePack,
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
        Ok(Self {
            pool,
            graph_id,
            rule_pack: Arc::new(rule_pack),
        })
    }

    #[cfg(test)]
    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn sync(&self, batch: &SyncBatch) -> Result<SyncSummary, StorageError> {
        let mut context = SyncContext::snapshot(&self.graph_id, "all");
        context.items_seen =
            u64::try_from(batch.sitemap.len() + batch.issues.len()).unwrap_or(u64::MAX);
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
        let rule_pack = Arc::clone(&self.rule_pack);

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
            .bind(serde_json::to_string(&SyncEvidenceSummary {
                sitemap_items: batch.sitemap.len(),
                issue_items: batch.issues.len(),
            })?)
            .execute(&mut *transaction)
            .await?;
        let mut upserted_nodes = 0_u64;
        let mut upserted_edges = 0_u64;
        for observation in &batch.sitemap {
            let normalized =
                url::normalize(&observation.url).map_err(StorageError::InvalidInput)?;
            let method = observation.method.to_ascii_uppercase();
            let (path_template, is_template) = url::parameterize_path(&normalized.path);
            let endpoint_hash = fingerprint::stable_id(
                "endpoint",
                &[&normalized.origin, &method, &normalized.path],
            );
            let endpoint = nodes::node(
                NodeKind::Endpoint,
                endpoint_hash.clone(),
                now,
                NodeMetadata {
                    origin: normalized.origin,
                    method,
                    path: normalized.path,
                    status: Some(observation.status),
                    content_type: headers::content_type(&observation.content_type),
                    response_fingerprint: fingerprint::response(&observation.response_body),
                    parameter_names: normalized.parameter_names,
                    path_template,
                    is_template,
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &endpoint,
                nodes::SearchFields {
                    origin: endpoint.metadata.origin.as_str(),
                    method: endpoint.metadata.method.as_str(),
                    path: endpoint.metadata.path.as_str(),
                    name: "",
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            upserted_nodes += 1;
            let origin = nodes::node(
                NodeKind::Origin,
                fingerprint::stable_id("origin", &[endpoint.metadata.origin.as_str()]),
                now,
                NodeMetadata {
                    origin: endpoint.metadata.origin.clone(),
                    ..NodeMetadata::default()
                },
            );
            for (surface, direction, payload) in [
                (
                    "request_message",
                    "request",
                    observation.request_bytes.as_slice(),
                ),
                (
                    "response_message",
                    "response",
                    observation.response_bytes.as_slice(),
                ),
            ] {
                if payload.is_empty() {
                    continue;
                }
                let blob_id = upsert_evidence_blob(
                    &mut transaction,
                    &endpoint.id,
                    surface,
                    direction,
                    &observation.content_type,
                    payload,
                    now,
                )
                .await?;
                persist_rule_findings(
                    &mut transaction,
                    &endpoint.id,
                    &blob_id,
                    surface,
                    payload,
                    rule_pack.as_ref(),
                    now,
                )
                .await?;
            }
            if !observation.response_body.is_empty() {
                let blob_id = upsert_evidence_blob(
                    &mut transaction,
                    &endpoint.id,
                    "response_body",
                    "response",
                    &observation.content_type,
                    &observation.response_body,
                    now,
                )
                .await?;
                persist_rule_findings(
                    &mut transaction,
                    &endpoint.id,
                    &blob_id,
                    "response_body",
                    &observation.response_body,
                    rule_pack.as_ref(),
                    now,
                )
                .await?;
            }
            index_evidence_for_search(&mut transaction, &endpoint.id).await?;
            upsert_source_node(
                &mut transaction,
                &origin,
                nodes::SearchFields {
                    origin: endpoint.metadata.origin.as_str(),
                    ..nodes::SearchFields::default()
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            upsert_source_edge(
                &mut transaction,
                &origin.id,
                &endpoint.id,
                crate::model::EdgeKind::Contains,
                &evidence_id,
                context,
                &sync_id,
                now,
            )
            .await?;
            upserted_nodes += 1;
            upserted_edges += 1;
            let mut parent_id = origin.id.clone();
            let mut accumulated = String::new();
            for segment in endpoint
                .metadata
                .path
                .as_str()
                .split('/')
                .filter(|segment| !segment.is_empty())
            {
                accumulated.push('/');
                accumulated.push_str(segment);
                let segment_node = nodes::node(
                    NodeKind::PathSegment,
                    fingerprint::stable_id(
                        "path_segment",
                        &[endpoint.metadata.origin.as_str(), &accumulated],
                    ),
                    now,
                    NodeMetadata {
                        segment: segment.to_owned(),
                        path: accumulated.clone(),
                        ..NodeMetadata::default()
                    },
                );
                upsert_source_node(
                    &mut transaction,
                    &segment_node,
                    nodes::SearchFields {
                        origin: endpoint.metadata.origin.as_str(),
                        path: segment_node.metadata.path.as_str(),
                        ..nodes::SearchFields::default()
                    },
                    context,
                    &sync_id,
                    now,
                )
                .await?;
                upsert_source_edge(
                    &mut transaction,
                    &parent_id,
                    &segment_node.id,
                    crate::model::EdgeKind::PathChild,
                    &evidence_id,
                    context,
                    &sync_id,
                    now,
                )
                .await?;
                parent_id = segment_node.id.clone();
                upserted_nodes += 1;
                upserted_edges += 1;
            }
            for parameter_name in &endpoint.metadata.parameter_names {
                let parameter = nodes::node(
                    NodeKind::Parameter,
                    fingerprint::stable_id("parameter", &[&endpoint.id, "query", parameter_name]),
                    now,
                    NodeMetadata {
                        name: parameter_name.clone(),
                        location: "query".to_owned(),
                        ..NodeMetadata::default()
                    },
                );
                upsert_source_node(
                    &mut transaction,
                    &parameter,
                    nodes::SearchFields {
                        name: parameter_name,
                        ..nodes::SearchFields::default()
                    },
                    context,
                    &sync_id,
                    now,
                )
                .await?;
                upsert_source_edge(
                    &mut transaction,
                    &endpoint.id,
                    &parameter.id,
                    crate::model::EdgeKind::AcceptsParameter,
                    &evidence_id,
                    context,
                    &sync_id,
                    now,
                )
                .await?;
                upserted_nodes += 1;
                upserted_edges += 1;
            }
            if let Some(response_hash) = endpoint.metadata.response_fingerprint.as_deref() {
                let response = nodes::node(
                    NodeKind::ResponseFingerprint,
                    fingerprint::stable_id("response_fingerprint", &[response_hash]),
                    now,
                    NodeMetadata {
                        fingerprint: response_hash.to_owned(),
                        content_type: endpoint.metadata.content_type.clone(),
                        ..NodeMetadata::default()
                    },
                );
                upsert_source_node(
                    &mut transaction,
                    &response,
                    nodes::SearchFields::default(),
                    context,
                    &sync_id,
                    now,
                )
                .await?;
                upsert_source_edge(
                    &mut transaction,
                    &endpoint.id,
                    &response.id,
                    crate::model::EdgeKind::RespondedWith,
                    &evidence_id,
                    context,
                    &sync_id,
                    now,
                )
                .await?;
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
                    NodeMetadata {
                        origin: target.origin,
                        method: "GET".to_owned(),
                        path: target.path,
                        ..NodeMetadata::default()
                    },
                );
                upsert_source_node(
                    &mut transaction,
                    &target_node,
                    nodes::SearchFields {
                        origin: target_node.metadata.origin.as_str(),
                        method: "GET",
                        path: target_node.metadata.path.as_str(),
                        name: "",
                    },
                    context,
                    &sync_id,
                    now,
                )
                .await?;
                upserted_nodes += 1;
                let kind = match relationship.kind {
                    "form" => crate::model::EdgeKind::FormSubmitsTo,
                    "script" => crate::model::EdgeKind::LoadsScript,
                    "redirect" => crate::model::EdgeKind::RedirectsTo,
                    "javascript_route" => crate::model::EdgeKind::DiscoversRoute,
                    _ => crate::model::EdgeKind::LinksTo,
                };
                upsert_source_edge(
                    &mut transaction,
                    &endpoint.id,
                    &target_node.id,
                    kind,
                    &evidence_id,
                    context,
                    &sync_id,
                    now,
                )
                .await?;
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
                NodeMetadata {
                    origin: normalized.origin,
                    method: "GET".to_owned(),
                    path: normalized.path,
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &endpoint,
                nodes::SearchFields {
                    origin: endpoint.metadata.origin.as_str(),
                    method: "GET",
                    path: endpoint.metadata.path.as_str(),
                    name: "",
                },
                context,
                &sync_id,
                now,
            )
            .await?;
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
                NodeMetadata {
                    name: issue.name.clone(),
                    severity: issue.severity.clone(),
                    confidence: issue.confidence.clone(),
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &issue_node,
                nodes::SearchFields {
                    name: &issue.name,
                    ..nodes::SearchFields::default()
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            upsert_source_edge(
                &mut transaction,
                &endpoint.id,
                &issue_node.id,
                crate::model::EdgeKind::HasIssue,
                &evidence_id,
                context,
                &sync_id,
                now,
            )
            .await?;
            upserted_nodes += 2;
            upserted_edges += 1;
        }
        for technology in &batch.technologies {
            let normalized =
                url::normalize(&technology.endpoint_url).map_err(StorageError::InvalidInput)?;
            let method = technology.method.to_ascii_uppercase();
            let endpoint_id = fingerprint::stable_id(
                "endpoint",
                &[&normalized.origin, &method, &normalized.path],
            );
            let endpoint = nodes::node(
                NodeKind::Endpoint,
                endpoint_id,
                now,
                NodeMetadata {
                    origin: normalized.origin,
                    method,
                    path: normalized.path,
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &endpoint,
                nodes::SearchFields {
                    origin: endpoint.metadata.origin.as_str(),
                    method: endpoint.metadata.method.as_str(),
                    path: endpoint.metadata.path.as_str(),
                    name: "",
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            let normalized_name = technology.name.trim().to_ascii_lowercase();
            let technology_node = nodes::node(
                NodeKind::Technology,
                fingerprint::stable_id("technology", &[&normalized_name]),
                now,
                NodeMetadata {
                    name: normalized_name,
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &technology_node,
                nodes::SearchFields {
                    name: technology_node.metadata.name.as_str(),
                    ..nodes::SearchFields::default()
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            upsert_source_edge(
                &mut transaction,
                &endpoint.id,
                &technology_node.id,
                crate::model::EdgeKind::HasTechnology,
                &evidence_id,
                context,
                &sync_id,
                now,
            )
            .await?;
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
                NodeMetadata {
                    origin: normalized.origin,
                    method: "GET".to_owned(),
                    path: normalized.path,
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &endpoint,
                nodes::SearchFields {
                    origin: endpoint.metadata.origin.as_str(),
                    method: "GET",
                    path: endpoint.metadata.path.as_str(),
                    name: "",
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            let kind = artifact.kind.trim().to_ascii_lowercase();
            let name = artifact.name.trim();
            let artifact_node = nodes::node(
                NodeKind::Artifact,
                fingerprint::stable_id("artifact", &[&kind, name, &artifact.fingerprint]),
                now,
                NodeMetadata {
                    kind,
                    name: name.to_owned(),
                    fingerprint: artifact.fingerprint.clone(),
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &artifact_node,
                nodes::SearchFields {
                    name: artifact_node.metadata.name.as_str(),
                    ..nodes::SearchFields::default()
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            upsert_source_edge(
                &mut transaction,
                &endpoint.id,
                &artifact_node.id,
                crate::model::EdgeKind::HasArtifact,
                &evidence_id,
                context,
                &sync_id,
                now,
            )
            .await?;
            upserted_nodes += 2;
            upserted_edges += 1;
        }
        for message in &batch.websocket_messages {
            let channel_id = fingerprint::stable_id("websocket_channel", &[&message.web_socket_id]);
            let channel = nodes::node(
                NodeKind::Artifact,
                channel_id.clone(),
                now,
                NodeMetadata {
                    artifact_kind: "websocket_channel".to_owned(),
                    web_socket_id: message.web_socket_id.clone(),
                    upgrade_url: message.upgrade_url.clone(),
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &channel,
                nodes::SearchFields {
                    name: &message.web_socket_id,
                    ..nodes::SearchFields::default()
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            let node_id =
                fingerprint::stable_id("websocket_message", &[&message.web_socket_id, &message.id]);
            let node = nodes::node(
                NodeKind::Artifact,
                node_id.clone(),
                now,
                NodeMetadata {
                    artifact_kind: "websocket_message".to_owned(),
                    web_socket_id: message.web_socket_id.clone(),
                    direction: message.direction.clone(),
                    upgrade_url: message.upgrade_url.clone(),
                    ..NodeMetadata::default()
                },
            );
            upsert_source_node(
                &mut transaction,
                &node,
                nodes::SearchFields {
                    name: &message.web_socket_id,
                    ..nodes::SearchFields::default()
                },
                context,
                &sync_id,
                now,
            )
            .await?;
            upsert_source_edge(
                &mut transaction,
                &channel_id,
                &node_id,
                crate::model::EdgeKind::HasMessage,
                &evidence_id,
                context,
                &sync_id,
                now,
            )
            .await?;
            for (surface, payload) in [
                ("websocket_payload", message.payload.as_slice()),
                (
                    "websocket_edited_payload",
                    message.edited_payload.as_slice(),
                ),
            ] {
                if payload.is_empty() {
                    continue;
                }
                let blob_id = upsert_evidence_blob(
                    &mut transaction,
                    &node_id,
                    surface,
                    &message.direction,
                    "application/octet-stream",
                    payload,
                    now,
                )
                .await?;
                persist_rule_findings(
                    &mut transaction,
                    &node_id,
                    &blob_id,
                    surface,
                    payload,
                    rule_pack.as_ref(),
                    now,
                )
                .await?;
            }
            index_evidence_for_search(&mut transaction, &node_id).await?;
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

    pub async fn checkpoint(
        &self,
        source: &str,
        scope: &str,
    ) -> Result<Option<(String, SyncCoverage)>, StorageError> {
        sqlx::query(
            "SELECT last_snapshot_id, coverage_json FROM source_checkpoints
             WHERE graph_id=?1 AND source=?2 AND scope=?3",
        )
        .bind(&self.graph_id)
        .bind(source)
        .bind(scope)
        .fetch_optional(&self.pool)
        .await?
        .map(|row| {
            Ok((
                row.get::<String, _>("last_snapshot_id"),
                serde_json::from_str::<SyncCoverage>(&row.get::<String, _>("coverage_json"))?,
            ))
        })
        .transpose()
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
        let last_synced_at =
            sqlx::query("SELECT value FROM graph_metadata WHERE key='last_synced_at'")
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
            freshness: if coverage.complete {
                "fresh"
            } else {
                "partial"
            }
            .to_owned(),
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
                evidence: EvidenceSource::default(),
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
            let metadata: NodeMetadata = serde_json::from_str(&row.get::<String, _>("metadata"))?;
            items.push(endpoint_from_metadata(
                row.get("id"),
                row.get("updated_at"),
                metadata,
            ));
        }
        let next = cursor + items.len() as u64;
        Ok(EndpointPage {
            items,
            total,
            truncated: next < total,
            next_cursor: (next < total).then_some(next),
            last_synced_at: self.status().await?.last_synced_at,
            evidence: EvidenceSource {
                source: Some("SQLite FTS5 node metadata".to_owned()),
            },
        })
    }

    pub async fn search_history(
        &self,
        query: &str,
        source: Option<&str>,
        cursor: u64,
        limit: u64,
    ) -> Result<crate::model::HistorySearchPage, StorageError> {
        let limit = validated_limit(limit)?;
        let pattern = literal_prefix_pattern(query);
        if pattern.is_empty() {
            return Ok(crate::model::HistorySearchPage {
                items: Vec::new(),
                total: 0,
                truncated: false,
                next_cursor: None,
            });
        }
        let source = source.filter(|value| !value.is_empty() && *value != "all");
        if let Some(value) = source
            && value != "http"
            && value != "websocket"
        {
            return Err(StorageError::InvalidInput(
                "history source must be http, websocket, or all".to_owned(),
            ));
        }
        let total = sqlx::query("SELECT count(*) AS count FROM history_search WHERE history_search MATCH ?1 AND (?2 IS NULL OR source=?2)")
            .bind(&pattern).bind(source).fetch_one(&self.pool).await?.get::<i64, _>("count") as u64;
        let rows = sqlx::query(
            "SELECT h.blob_id, h.node_id, h.source, h.surface, h.direction, h.content_type, h.url, h.method, snippet(history_search, 8, '[', ']', ' … ', 24) AS snippet, e.byte_length
             FROM history_search h JOIN evidence_blobs e ON e.id=h.blob_id
             WHERE history_search MATCH ?1 AND (?2 IS NULL OR h.source=?2)
             ORDER BY bm25(history_search), h.rowid LIMIT ?3 OFFSET ?4",
        ).bind(&pattern).bind(source).bind(limit as i64).bind(cursor as i64).fetch_all(&self.pool).await?;
        let items = rows
            .into_iter()
            .map(|row| crate::model::HistorySearchHit {
                blob_id: row.get("blob_id"),
                node_id: row.get("node_id"),
                source: row.get("source"),
                surface: row.get("surface"),
                direction: row.get("direction"),
                content_type: row.get("content_type"),
                url: row.get("url"),
                method: row.get("method"),
                snippet: row.get("snippet"),
                byte_length: row.get::<i64, _>("byte_length") as u64,
            })
            .collect::<Vec<_>>();
        let end = cursor.saturating_add(items.len() as u64);
        Ok(crate::model::HistorySearchPage {
            items,
            total,
            truncated: end < total,
            next_cursor: (end < total).then_some(end),
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
        let metadata: NodeMetadata = serde_json::from_str(&row.get::<String, _>("metadata"))?;
        Ok(Some(endpoint_from_metadata(
            row.get("id"),
            row.get("updated_at"),
            metadata,
        )))
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
            .map(|row| {
                Ok(Neighbor {
                    edge_id: row.get("edge_id"),
                    kind: row.get("kind"),
                    direction: if row.get::<String, _>("from_id") == node_id {
                        "outgoing".to_owned()
                    } else {
                        "incoming".to_owned()
                    },
                    node_id: row.get("node_id"),
                    node_kind: row.get("node_kind"),
                    metadata: serde_json::from_str(&row.get::<String, _>("metadata"))?,
                })
            })
            .collect::<Result<Vec<_>, StorageError>>()?;
        let next = cursor + items.len() as u64;
        Ok(NeighborPage {
            items,
            total,
            truncated: next < total,
            next_cursor: (next < total).then_some(next),
            last_synced_at: self.status().await?.last_synced_at,
            evidence: EvidenceSource {
                source: Some("SQLite adjacency edges".to_owned()),
            },
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

    pub async fn export_exact_json(
        &self,
        cursor: u64,
        limit: u64,
    ) -> Result<crate::export::json::JsonExport, StorageError> {
        let limit = validated_limit(limit)?;
        crate::export::json::exact_page(
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
    pub async fn shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Result<crate::ShortestPath, StorageError> {
        analysis::shortest_path(&self.pool, from_id, to_id, max_depth).await
    }

    pub async fn endpoint_clusters(
        &self,
        limit: usize,
    ) -> Result<Vec<crate::Cluster>, StorageError> {
        analysis::clusters(&self.pool, limit).await
    }

    pub async fn impact(
        &self,
        start_id: &str,
        max_depth: usize,
        limit: usize,
    ) -> Result<Vec<crate::ImpactNode>, StorageError> {
        analysis::impact(&self.pool, start_id, max_depth, limit).await
    }

    pub async fn import_openapi(
        &self,
        content: &str,
        base_url: &str,
    ) -> Result<SyncSummary, StorageError> {
        let observations =
            crate::ingest::openapi::observations(content.as_bytes(), base_url, 16_384)
                .map_err(StorageError::InvalidInput)?;
        let batch = SyncBatch {
            sitemap: observations,
            ..SyncBatch::default()
        };
        let mut context = SyncContext::snapshot(&self.graph_id, "openapi");
        context.source = "openapi_import".to_string();
        context.items_seen = batch.sitemap.len() as u64;
        self.sync_with_context(&batch, &context).await
    }

    pub async fn security_view(
        &self,
        view_name: &str,
        limit: usize,
    ) -> Result<serde_json::Value, StorageError> {
        analysis::security_view(&self.pool, view_name, limit).await
    }
}

async fn index_evidence_for_search(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    node_id: &str,
) -> Result<(), StorageError> {
    let metadata = sqlx::query("SELECT metadata FROM nodes WHERE id=?1")
        .bind(node_id)
        .fetch_one(&mut **transaction)
        .await?
        .get::<String, _>("metadata");
    let metadata: NodeMetadata = serde_json::from_str(&metadata)?;
    let url = if metadata.upgrade_url.is_empty() {
        metadata.url.as_str()
    } else {
        metadata.upgrade_url.as_str()
    };
    let method = metadata.method.as_str();
    let blobs = sqlx::query("SELECT id, surface, direction, content_type, payload FROM evidence_blobs WHERE source_entry_id=?1")
        .bind(node_id).fetch_all(&mut **transaction).await?;
    for row in blobs {
        let content_type = row.get::<String, _>("content_type");
        let payload = row.get::<Vec<u8>, _>("payload");
        let text = match std::str::from_utf8(&payload) {
            Ok(value) => value.to_owned(),
            Err(_) => {
                if content_type.starts_with("image/")
                    || content_type.starts_with("video/")
                    || content_type.starts_with("audio/")
                    || content_type.contains("octet-stream")
                {
                    format!("[binary media: {} bytes]", payload.len())
                } else if payload.len() > 64 * 1024 {
                    format!("[large binary: {} bytes]", payload.len())
                } else {
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, payload)
                }
            }
        };
        let surface = row.get::<String, _>("surface");
        let blob_id = row.get::<String, _>("id");
        sqlx::query("DELETE FROM history_search WHERE blob_id=?1")
            .bind(&blob_id)
            .execute(&mut **transaction)
            .await?;
        sqlx::query("INSERT INTO history_search(blob_id, node_id, source, surface, direction, content_type, url, method, payload) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
            .bind(blob_id).bind(node_id)
            .bind(if surface.starts_with("websocket_") { "websocket" } else { "http" })
            .bind(surface).bind(row.get::<String, _>("direction")).bind(row.get::<String, _>("content_type"))
            .bind(url).bind(method).bind(text).execute(&mut **transaction).await?;
    }
    Ok(())
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
        SiteGraph {
            pool,
            graph_id: "test".to_owned(),
            rule_pack: std::sync::Arc::new(RulePack::default_exact().unwrap()),
        }
    }

    fn observation(path: &str, body: &[u8]) -> SitemapObservation {
        SitemapObservation {
            url: format!("https://example.test/{path}?token=secret"),
            method: "GET".to_owned(),
            status: 200,
            content_type: "text/html".to_owned(),
            response_body: body.to_vec(),
            request_bytes: Vec::new(),
            response_bytes: Vec::new(),
            redirect_url: String::new(),
            response_links: Vec::new(),
            form_actions: Vec::new(),
            script_sources: Vec::new(),
        }
    }

    #[tokio::test]
    async fn large_source_batch_remains_bounded_by_page_commits() {
        let graph = graph().await;
        for page in 0..21 {
            let sitemap = (0..500)
                .map(|item| observation(&format!("page-{page}-item-{item}"), b""))
                .collect::<Vec<_>>();
            let mut context = SyncContext::snapshot("test", "stress");
            context.run_id = "stress-run".to_owned();
            context.pages_seen = page + 1;
            context.items_seen = (page + 1) * 500;
            context.cursor = (!context.complete).then(|| format!("cursor-{}", page + 1));
            graph
                .sync_with_context(
                    &SyncBatch {
                        sitemap,
                        ..SyncBatch::default()
                    },
                    &context,
                )
                .await
                .unwrap();
        }
        let status = graph.status().await.unwrap();
        assert!(status.total_nodes > 10_000);
        assert!(status.coverage.complete);
        assert_eq!(status.coverage.items_indexed, 10_500);
        assert_eq!(status.coverage.pages_read, 21);
    }

    #[tokio::test]
    async fn project_graph_ids_remain_isolated_in_separate_databases() {
        let directory = tempfile::tempdir().unwrap();
        let graph_a = SiteGraph::open_with_id(&directory.path().join("a.sqlite"), "project-a")
            .await
            .unwrap();
        let graph_b = SiteGraph::open_with_id(&directory.path().join("b.sqlite"), "project-b")
            .await
            .unwrap();
        graph_a
            .sync(&SyncBatch {
                sitemap: vec![observation("only-a", b"")],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        assert_eq!(graph_a.status().await.unwrap().graph_id, "project-a");
        assert_eq!(graph_b.status().await.unwrap().graph_id, "project-b");
        assert!(graph_a.status().await.unwrap().total_nodes > 0);
        assert_eq!(graph_b.status().await.unwrap().total_nodes, 0);
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
    async fn duplicate_findings_across_surfaces_update_existing_record() {
        let graph = graph().await;
        let payload = b"token=duplicate-marker";
        let mut item = observation("duplicate-finding", payload);
        item.response_bytes = payload.to_vec();
        graph
            .sync(&SyncBatch {
                sitemap: vec![item],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        let node_id = fingerprint::stable_id(
            "endpoint",
            &["https://example.test", "GET", "/duplicate-finding"],
        );
        let findings = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM enrichment_findings WHERE node_id=?1",
        )
        .bind(node_id)
        .fetch_one(graph.pool())
        .await
        .unwrap();
        assert_eq!(findings, 1);
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
                    method: "GET".to_owned(),
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
    async fn sync_reuses_migrated_edge_with_legacy_primary_id() {
        let graph = graph().await;
        let endpoint = observation("legacy-edge", b"");
        graph
            .sync(&SyncBatch {
                sitemap: vec![endpoint.clone()],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        let existing =
            sqlx::query("SELECT id, from_id, to_id, kind FROM edges WHERE kind='contains' LIMIT 1")
                .fetch_one(graph.pool())
                .await
                .unwrap();
        let stable_id = existing.get::<String, _>("id");
        let legacy_id = format!("legacy-{stable_id}");
        let evidence_id =
            sqlx::query_scalar::<_, String>("SELECT evidence_id FROM edges WHERE id=?1")
                .bind(&stable_id)
                .fetch_one(graph.pool())
                .await
                .unwrap();
        sqlx::query("DELETE FROM edge_evidence WHERE edge_id=?1")
            .bind(&stable_id)
            .execute(graph.pool())
            .await
            .unwrap();
        sqlx::query("DELETE FROM source_edges WHERE edge_id=?1")
            .bind(&stable_id)
            .execute(graph.pool())
            .await
            .unwrap();
        sqlx::query("DELETE FROM edges WHERE id=?1")
            .bind(&stable_id)
            .execute(graph.pool())
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO edges(id, from_id, to_id, kind, evidence_id, created_at, updated_at, metadata)
             VALUES(?1, ?2, ?3, ?4, ?5, 1, 1, '{}')",
        )
        .bind(&legacy_id)
        .bind(existing.get::<String, _>("from_id"))
        .bind(existing.get::<String, _>("to_id"))
        .bind(existing.get::<String, _>("kind"))
        .bind(evidence_id)
        .execute(graph.pool())
        .await
        .unwrap();

        graph
            .sync(&SyncBatch {
                sitemap: vec![endpoint],
                ..SyncBatch::default()
            })
            .await
            .unwrap();

        let stored_id = sqlx::query_scalar::<_, String>(
            "SELECT id FROM edges WHERE from_id=?1 AND to_id=?2 AND kind=?3",
        )
        .bind(existing.get::<String, _>("from_id"))
        .bind(existing.get::<String, _>("to_id"))
        .bind(existing.get::<String, _>("kind"))
        .fetch_one(graph.pool())
        .await
        .unwrap();
        assert_eq!(stored_id, legacy_id);
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

    #[tokio::test]
    async fn diff_reports_nodes_edges_and_tombstones_without_sql_errors() {
        let graph = graph().await;
        let mut first = SyncContext::snapshot("test", "all");
        first.run_id = "diff-first".to_owned();
        first.items_seen = 2;
        graph
            .sync_with_context(
                &SyncBatch {
                    sitemap: vec![observation("kept", b""), observation("gone", b"")],
                    ..SyncBatch::default()
                },
                &first,
            )
            .await
            .unwrap();
        let mut second = SyncContext::snapshot("test", "all");
        second.run_id = "diff-second".to_owned();
        second.items_seen = 1;
        graph
            .sync_with_context(
                &SyncBatch {
                    sitemap: vec![observation("kept", b"")],
                    ..SyncBatch::default()
                },
                &second,
            )
            .await
            .unwrap();
        let diff = graph.diff(0, 0, 500).await.unwrap();
        assert!(!diff.added_node_ids.is_empty());
        assert!(!diff.added_edge_ids.is_empty());
        assert!(!diff.removed_node_ids.is_empty());
        assert!(!diff.removed_edge_ids.is_empty());
    }

    #[tokio::test]
    async fn export_profiles_separate_metadata_from_exact_payloads() {
        let graph = graph().await;
        graph
            .sync(&SyncBatch {
                sitemap: vec![observation("profiles", b"token=exact-export-value")],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        let metadata = serde_json::to_string(&graph.export_json(0, 50).await.unwrap()).unwrap();
        let exact = serde_json::to_string(&graph.export_exact_json(0, 50).await.unwrap()).unwrap();
        assert!(!metadata.contains("exact-export-value"));
        assert!(exact.contains("payload_base64"));
        assert!(exact.contains("exact"));
    }

    #[tokio::test]
    async fn websocket_payload_evidence_round_trips_exactly() {
        let graph = graph().await;
        graph
            .sync(&SyncBatch {
                websocket_messages: vec![crate::model::WebSocketObservation {
                    id: "1".to_owned(),
                    web_socket_id: "socket-1".to_owned(),
                    direction: "CLIENT_TO_SERVER".to_owned(),
                    upgrade_url: "wss://example.test/socket".to_owned(),
                    payload: b"binary\0payload\xff".to_vec(),
                    edited_payload: Vec::new(),
                }],
                ..SyncBatch::default()
            })
            .await
            .unwrap();
        let payload =
            sqlx::query("SELECT payload FROM evidence_blobs WHERE surface='websocket_payload'")
                .fetch_one(graph.pool())
                .await
                .unwrap()
                .get::<Vec<u8>, _>("payload");
        assert_eq!(payload, b"binary\0payload\xff");
    }
    #[tokio::test]
    async fn javascript_routes_and_websocket_channels_persist_with_provenance_edges() {
        let graph = graph().await;
        let mut script = observation("app.js", b"fetch('/api/admin');");
        script.content_type = "application/javascript".to_owned();
        graph
            .sync(&SyncBatch {
                sitemap: vec![script],
                websocket_messages: vec![crate::model::WebSocketObservation {
                    id: "message-1".to_owned(),
                    web_socket_id: "socket-1".to_owned(),
                    direction: "server_to_client".to_owned(),
                    upgrade_url: "wss://example.test/socket".to_owned(),
                    payload: b"token=socket-secret".to_vec(),
                    edited_payload: Vec::new(),
                }],
                ..SyncBatch::default()
            })
            .await
            .unwrap();

        let route_edges =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM edges WHERE kind='discovers_route'")
                .fetch_one(graph.pool())
                .await
                .unwrap();
        let channel_edges =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM edges WHERE kind='has_message'")
                .fetch_one(graph.pool())
                .await
                .unwrap();
        let channel_nodes = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM nodes WHERE json_extract(metadata, '$.artifact_kind')='websocket_channel'",
        )
        .fetch_one(graph.pool())
        .await
        .unwrap();
        assert_eq!(route_edges, 1);
        assert_eq!(channel_edges, 1);
        assert_eq!(channel_nodes, 1);
    }
    #[tokio::test]
    async fn history_search_indexes_http_and_websocket_payloads() {
        let graph = graph().await;
        let mut http = observation("search", b"");
        http.request_bytes = b"GET /search HTTP/1.1\r\nX-Trace: alpha-needle\r\n\r\n".to_vec();
        http.response_bytes = b"HTTP/1.1 200 OK\r\n\r\nbeta-needle".to_vec();
        let batch = SyncBatch {
            sitemap: vec![http],
            websocket_messages: vec![crate::model::WebSocketObservation {
                id: "7".to_owned(),
                web_socket_id: "3".to_owned(),
                direction: "CLIENT_TO_SERVER".to_owned(),
                upgrade_url: "wss://example.test/socket".to_owned(),
                payload: b"gamma-needle".to_vec(),
                edited_payload: Vec::new(),
            }],
            ..SyncBatch::default()
        };
        graph.sync(&batch).await.unwrap();
        let indexed_before = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM history_search")
            .fetch_one(graph.pool())
            .await
            .unwrap();
        graph.sync(&batch).await.unwrap();
        let indexed_after = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM history_search")
            .fetch_one(graph.pool())
            .await
            .unwrap();
        assert_eq!(indexed_after, indexed_before);

        let http_page = graph
            .search_history("alpha-needle", Some("http"), 0, 10)
            .await
            .unwrap();
        assert_eq!(http_page.total, 1);
        assert_eq!(http_page.items[0].source, "http");
        let websocket_page = graph
            .search_history("gamma-needle", Some("websocket"), 0, 10)
            .await
            .unwrap();
        assert_eq!(websocket_page.total, 1);
        assert_eq!(websocket_page.items[0].source, "websocket");
        assert_eq!(
            graph
                .search_history("gamma-needle", Some("http"), 0, 10)
                .await
                .unwrap()
                .total,
            0
        );
    }
}
