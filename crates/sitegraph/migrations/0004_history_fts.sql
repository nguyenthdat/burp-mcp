CREATE VIRTUAL TABLE history_search USING fts5(
  blob_id UNINDEXED,
  node_id UNINDEXED,
  source,
  surface,
  direction,
  content_type,
  url,
  method,
  payload
);
-- Existing evidence is backfilled here. New evidence is inserted by the storage layer so binary
-- payloads can be converted to UTF-8 or Base64 before reaching FTS5.
INSERT INTO history_search(blob_id, node_id, source, surface, direction, content_type, url, method, payload)
SELECT
  e.id,
  e.source_entry_id,
  CASE WHEN e.surface LIKE 'websocket_%' THEN 'websocket' ELSE 'http' END,
  e.surface,
  COALESCE(e.direction, ''),
  COALESCE(e.content_type, ''),
  COALESCE(json_extract(n.metadata, '$.upgrade_url'), json_extract(n.metadata, '$.url'), ''),
  COALESCE(json_extract(n.metadata, '$.method'), ''),
  CAST(e.payload AS TEXT)
FROM evidence_blobs e
JOIN nodes n ON n.id = e.source_entry_id;
