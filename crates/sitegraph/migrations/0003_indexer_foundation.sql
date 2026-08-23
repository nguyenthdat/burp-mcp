CREATE TABLE edges_v3 (
  id TEXT PRIMARY KEY,
  from_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  to_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  evidence_id TEXT NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  metadata TEXT NOT NULL CHECK(json_valid(metadata)),
  UNIQUE(from_id, to_id, kind)
);

INSERT OR IGNORE INTO edges_v3(
  id, from_id, to_id, kind, evidence_id, created_at, updated_at, metadata
)
SELECT id, from_id, to_id, kind, evidence_id, created_at, created_at, metadata
FROM edges
ORDER BY created_at, id;

DROP TABLE edges;
ALTER TABLE edges_v3 RENAME TO edges;

CREATE INDEX edges_from_kind_to ON edges(from_id, kind, to_id, id);
CREATE INDEX edges_to_kind_from ON edges(to_id, kind, from_id, id);
CREATE INDEX edges_updated_id ON edges(updated_at, id);

CREATE TABLE edge_evidence (
  edge_id TEXT NOT NULL REFERENCES edges(id) ON DELETE CASCADE,
  evidence_id TEXT NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
  first_seen_at INTEGER NOT NULL,
  last_seen_at INTEGER NOT NULL,
  PRIMARY KEY(edge_id, evidence_id)
);

INSERT INTO edge_evidence(edge_id, evidence_id, first_seen_at, last_seen_at)
SELECT id, evidence_id, created_at, updated_at FROM edges;

CREATE TABLE source_checkpoints (
  graph_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  last_cursor TEXT,
  last_snapshot_id TEXT,
  last_success_at INTEGER,
  coverage_json TEXT NOT NULL CHECK(json_valid(coverage_json)),
  PRIMARY KEY(graph_id, source, scope)
);

CREATE TABLE sync_runs (
  id TEXT PRIMARY KEY,
  graph_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  finished_at INTEGER,
  status TEXT NOT NULL,
  complete INTEGER NOT NULL DEFAULT 0,
  items_seen INTEGER NOT NULL DEFAULT 0,
  pages_seen INTEGER NOT NULL DEFAULT 0,
  error TEXT
);
CREATE INDEX sync_runs_graph_started ON sync_runs(graph_id, started_at DESC, id);

CREATE TABLE source_nodes (
  graph_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  last_seen_run_id TEXT NOT NULL REFERENCES sync_runs(id) ON DELETE CASCADE,
  last_seen_at INTEGER NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY(graph_id, source, scope, node_id)
);
CREATE INDEX source_nodes_active ON source_nodes(graph_id, source, scope, active, node_id);

CREATE TABLE source_edges (
  graph_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  edge_id TEXT NOT NULL REFERENCES edges(id) ON DELETE CASCADE,
  last_seen_run_id TEXT NOT NULL REFERENCES sync_runs(id) ON DELETE CASCADE,
  last_seen_at INTEGER NOT NULL,
  active INTEGER NOT NULL DEFAULT 1,
  PRIMARY KEY(graph_id, source, scope, edge_id)
);
CREATE INDEX source_edges_active ON source_edges(graph_id, source, scope, active, edge_id);

CREATE TABLE tombstones (
  graph_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  first_missing_at INTEGER NOT NULL,
  last_confirmed_at INTEGER NOT NULL,
  reason TEXT NOT NULL,
  PRIMARY KEY(graph_id, entity_type, entity_id, source, scope)
);
CREATE INDEX tombstones_confirmed ON tombstones(graph_id, last_confirmed_at, entity_type, entity_id);

CREATE TABLE evidence_blobs (
  id TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL,
  blake3 TEXT NOT NULL,
  source_entry_id TEXT,
  surface TEXT NOT NULL,
  direction TEXT,
  content_type TEXT,
  payload BLOB NOT NULL,
  byte_length INTEGER NOT NULL,
  observed_at INTEGER NOT NULL,
  UNIQUE(sha256, surface, direction)
);

CREATE TABLE enrichment_findings (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  evidence_blob_id TEXT NOT NULL REFERENCES evidence_blobs(id),
  enricher_id TEXT NOT NULL,
  enricher_version TEXT NOT NULL,
  ruleset_id TEXT,
  ruleset_version TEXT,
  input_fingerprint TEXT NOT NULL,
  kind TEXT NOT NULL,
  severity TEXT,
  confidence REAL,
  byte_start INTEGER NOT NULL,
  byte_end INTEGER NOT NULL,
  capture BLOB NOT NULL,
  incomplete INTEGER NOT NULL DEFAULT 0,
  limit_reason TEXT,
  metadata TEXT NOT NULL CHECK(json_valid(metadata)),
  observed_at INTEGER NOT NULL,
  UNIQUE(node_id, enricher_id, ruleset_id, input_fingerprint, byte_start, byte_end)
);

CREATE INDEX nodes_updated_id ON nodes(updated_at, id);
