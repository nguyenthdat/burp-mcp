CREATE TABLE nodes (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  stable_hash TEXT NOT NULL UNIQUE,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  metadata TEXT NOT NULL CHECK(json_valid(metadata))
);

CREATE TABLE evidence (
  id TEXT PRIMARY KEY,
  source TEXT NOT NULL,
  observed_at INTEGER NOT NULL,
  summary TEXT NOT NULL CHECK(json_valid(summary))
);

CREATE TABLE edges (
  id TEXT PRIMARY KEY,
  from_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  to_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  evidence_id TEXT NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  metadata TEXT NOT NULL CHECK(json_valid(metadata)),
  UNIQUE(from_id, to_id, kind, evidence_id)
);

CREATE TABLE graph_metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE INDEX nodes_kind_id ON nodes(kind, id);
CREATE INDEX edges_from_kind_to ON edges(from_id, kind, to_id, id);
CREATE INDEX edges_to_kind_from ON edges(to_id, kind, from_id, id);
