use crate::enrichment::RulePack;
use crate::normalize::fingerprint;
use crate::storage::StorageError;
use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};

#[derive(serde::Serialize)]
struct FindingMetadata<'a> {
    capture_policy: &'static str,
    surface: &'a str,
    rule_pack: &'a str,
}

pub(super) async fn upsert_evidence_blob(
    transaction: &mut Transaction<'_, Sqlite>,
    source_entry_id: &str,
    surface: &str,
    direction: &str,
    content_type: &str,
    payload: &[u8],
    observed_at: i64,
) -> Result<String, StorageError> {
    let blake3_digest = blake3::hash(payload).to_hex().to_string();
    let sha256_digest = format!("{:x}", Sha256::digest(payload));
    let id = fingerprint::stable_id("evidence_blob", &[surface, &blake3_digest]);
    sqlx::query(
        "INSERT OR IGNORE INTO evidence_blobs(id, sha256, blake3, source_entry_id, surface, direction, content_type, payload, byte_length, observed_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)", 
    )
    .bind(&id)
    .bind(&sha256_digest)
    .bind(&blake3_digest)
    .bind(source_entry_id)
    .bind(surface)
    .bind(direction)
    .bind(content_type)
    .bind(payload)
    .bind(i64::try_from(payload.len()).unwrap_or(i64::MAX))
    .bind(observed_at)
    .execute(&mut **transaction)
    .await?;
    Ok(id)
}

pub(super) async fn persist_rule_findings(
    transaction: &mut Transaction<'_, Sqlite>,
    node_id: &str,
    blob_id: &str,
    surface: &str,
    payload: &[u8],
    rule_pack: &RulePack,
    observed_at: i64,
) -> Result<(), StorageError> {
    let body_hash = blake3::hash(payload).to_hex().to_string();
    let stored = sqlx::query("SELECT payload, blake3 FROM evidence_blobs WHERE id=?1")
        .bind(blob_id)
        .fetch_one(&mut **transaction)
        .await?;
    let stored_bytes = stored.get::<Vec<u8>, _>("payload");
    let stored_hash = stored.get::<String, _>("blake3");
    if blake3::hash(&stored_bytes).to_hex().to_string() != stored_hash {
        return Err(StorageError::InvalidInput(format!(
            "evidence blob {blob_id} failed integrity verification"
        )));
    }
    for finding in rule_pack.matches(surface, payload) {
        let finding_id = fingerprint::stable_id(
            "exact_finding",
            &[
                node_id,
                blob_id,
                finding.rule_id.as_str(),
                &finding.byte_start.to_string(),
                &finding.byte_end.to_string(),
            ],
        );
        sqlx::query(
            "INSERT INTO enrichment_findings(id, node_id, evidence_blob_id, enricher_id, enricher_version, ruleset_id, ruleset_version, input_fingerprint, kind, severity, confidence, byte_start, byte_end, capture, incomplete, limit_reason, metadata, observed_at)
             VALUES(?1, ?2, ?3, 'default_rule_pack', ?4, ?5, ?6, ?7, ?8, ?9, 0.8, ?10, ?11, ?12, 0, NULL, ?13, ?14)
             ON CONFLICT(node_id, enricher_id, ruleset_id, input_fingerprint, byte_start, byte_end) DO UPDATE SET
               evidence_blob_id=excluded.evidence_blob_id,
               enricher_version=excluded.enricher_version,
               ruleset_version=excluded.ruleset_version,
               input_fingerprint=excluded.input_fingerprint,
               kind=excluded.kind,
               severity=excluded.severity,
               confidence=excluded.confidence,
               capture=excluded.capture,
               incomplete=excluded.incomplete,
               limit_reason=excluded.limit_reason,
               metadata=excluded.metadata,
               observed_at=excluded.observed_at",
        )
        .bind(&finding_id)
        .bind(node_id)
        .bind(blob_id)
        .bind(rule_pack.version())
        .bind(rule_pack.id())
        .bind(rule_pack.version())
        .bind(&body_hash)
        .bind(&finding.rule_id)
        .bind(&finding.severity)
        .bind(i64::try_from(finding.byte_start).unwrap_or(i64::MAX))
        .bind(i64::try_from(finding.byte_end).unwrap_or(i64::MAX))
        .bind(&finding.capture)
        .bind(serde_json::to_string(&FindingMetadata {
            capture_policy: "exact",
            surface,
            rule_pack: rule_pack.id(),
        })?)
        .bind(observed_at)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
