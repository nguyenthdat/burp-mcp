@0xd8d7626a5badfd64;

struct CheckpointTask {
  source @0 :Text;
  scope @1 :Text;
}

struct SearchTask {
  query @0 :Text;
  cursor @1 :UInt64;
  limit @2 :UInt64;
}

struct SearchHistoryTask {
  query @0 :Text;
  source @1 :Text;
  cursor @2 :UInt64;
  limit @3 :UInt64;
}

struct EndpointTask {
  id @0 :Text;
}

struct NeighborsTask {
  nodeId @0 :Text;
  cursor @1 :UInt64;
  limit @2 :UInt64;
}

struct DiffTask {
  since @0 :Int64;
  cursor @1 :UInt64;
  limit @2 :UInt64;
}

struct PageTask {
  cursor @0 :UInt64;
  limit @1 :UInt64;
}

struct TraceTask {
  startId @0 :Text;
  maxDepth @1 :UInt32;
  limit @2 :UInt32;
}

struct ShortestPathTask {
  fromId @0 :Text;
  toId @1 :Text;
  maxDepth @2 :UInt64;
}

struct LimitTask {
  limit @0 :UInt64;
}

struct ImpactTask {
  nodeId @0 :Text;
  maxDepth @1 :UInt64;
  limit @2 :UInt64;
}

struct Task {
  union {
    status @0 :Void;
    checkpoint @1 :CheckpointTask;
    syncWithContext @2 :Data;
    search @3 :SearchTask;
    searchHistory @4 :SearchHistoryTask;
    endpoint @5 :EndpointTask;
    neighbors @6 :NeighborsTask;
    trace @7 :TraceTask;
    shortestPath @8 :ShortestPathTask;
    diff @9 :DiffTask;
    exportJson @10 :PageTask;
    exportCsv @11 :PageTask;
    endpointClusters @12 :LimitTask;
    impact @13 :ImpactTask;
    exportExactJson @14 :PageTask;
  }
}

interface Sitegraph {
  call @0 (token :Text, task :Task) -> (ok :Bool, payload :Data, error :Text);
}
