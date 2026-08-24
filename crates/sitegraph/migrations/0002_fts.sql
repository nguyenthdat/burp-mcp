CREATE VIRTUAL TABLE node_search USING fts5(
  node_id UNINDEXED,
  kind,
  origin,
  method,
  path,
  name,
);
