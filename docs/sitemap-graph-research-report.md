# Sitemap Graph Research Report

## 0. Kết luận điều hành

**Khuyến nghị chính:** xây `SitegraphIndexer` trong process Rust, sở hữu một bounded queue và một graph writer duy nhất; bật auto-index bằng cấu hình opt-in; dùng snapshot có checkpoint làm nguồn đúng (source of truth), sau đó bổ sung event feed từ Kotlin để giảm độ trễ. Mỗi Burp project phải được ánh xạ sang đúng một SQLite database riêng. Project tạm dùng database với tên ngẫu nhiên không suy ra từ target. Không nên bắt đầu bằng một daemon dùng chung giữa nhiều process như `codebase-memory-mcp`: Burp MCP chỉ có một nguồn dữ liệu sống là Montoya/Burp session, và lifecycle của extension phải đóng worker, queue, RPC và database khi unload.

Auto-index phải đọc hết history hiện có, không có hard cap tổng 10.000, 100.000 hay một số item cố định khác. Giới hạn chỉ áp dụng cho từng page, từng transaction, bytes transient, queue và deadline để giữ backpressure; checkpoint tiếp tục sang page kế tiếp cho tới khi đạt end-of-source. Người dùng có thể hủy hoặc pause, nhưng indexer không được tự tuyên bố complete vì chạm một total-item cap.

Metadata enrichment là pipeline first-class. Ngoài normalize/HTML/OpenAPI, graph nên chạy các enricher versioned và bounded: nhận diện JavaScript component/vulnerability theo database kiểu Retire.js; tagging/extraction bằng rule packs regex theo mô hình HaE; technology fingerprint; secret-pattern metadata; GraphQL/OpenAPI/JS-route discovery. **Enrichment không redact dữ liệu match:** security-testing profile phải giữ exact capture, exact token/value, vị trí match và payload evidence đủ cho replay/debug. Enricher chỉ giới hạn kích thước, thời gian và phạm vi input; không biến kết quả thành placeholder hoặc hash thay cho dữ liệu kiểm thử.

MVP hiện tại đã có nền tảng đúng cho topology: SQLite/WAL, migration, FTS5, BLAKE3 stable IDs, bounded search/traversal/export và các tool `sitegraph_*`. Tuy nhiên, nó vẫn là **manual snapshot sync**, chưa phải auto-index pipeline; target enrichment sẽ bổ sung project-scoped exact evidence blobs.

1. **Checkpoint và evidence chưa đủ ổn định cho incremental sync.** Edge ID hiện phụ thuộc `evidence_id`; `evidence_id` lại phụ thuộc `sync_id` tạo từ timestamp. Snapshot lặp qua ranh giới giây có thể tạo edge mới dù quan hệ không đổi.
2. **Không có reconciliation/deletion model.** Upsert xử lý những gì nhìn thấy nhưng chưa có tombstone hoặc `last_seen_run` để biết entry đã biến mất khỏi Burp sitemap.
3. **Snapshot đang có hard cap và thiếu pagination đầy đủ.** Rust dừng ở 10.000 sitemap entries; scanner issues chỉ lấy một page 500 phần tử. Kotlin lấy toàn bộ site map vào memory trước khi `drop/take`. Target phải bỏ total-history cap và chỉ giữ page/resource bounds.
4. **Chưa có per-project database resolver.** CLI hiện mở một graph path; target phải dùng Montoya `Project.id()`/`name()` để chọn một DB riêng cho từng Burp project và random DB name cho project tạm.
5. **Chưa có watcher, queue, backpressure, trạng thái auto-index hoặc startup/reconnect orchestration.** `sitegraph_sync` chỉ chạy khi MCP client gọi tool.
6. **Graph query cần correctness hardening.** Trace recursive CTE chưa có visited-set/cycle guard; diff hiện chỉ trả node added/updated, chưa mô tả removed nodes hoặc edge changes.
7. **Ingestion/enrichment mới ở mức MVP.** HTML extraction dùng regex; OpenAPI ingestion hiện trả lỗi “not enabled”; chưa có versioned Retire.js-style component matcher, HaE-style rule packs hoặc re-enrichment lifecycle.

Định hướng này giữ đúng PLAN.md: Rust sở hữu indexing, persistence, enrichment và graph queries; Kotlin chỉ cung cấp Montoya adapter, project identity và typed gRPC; MCP không mở arbitrary SQL/Cypher.

---

## 1. Phạm vi và phương pháp nghiên cứu

### 1.1 Phạm vi

Báo cáo trả lời bốn câu hỏi:

- Sitemap graph hiện có những capability nào và thiếu gì để **auto-index** liên tục?
- Có thể học gì từ kiến trúc indexing/watch/persistence của [`DeusData/codebase-memory-mcp`](https://github.com/DeusData/codebase-memory-mcp)?
- Schema, ingestion, lifecycle, MCP tools và vận hành cần thay đổi thế nào?
- Roadmap nào giảm rủi ro nhưng vẫn tạo được giá trị sớm cho Burp security workflow?

### 1.2 Cấp độ bằng chứng

- **[Đã xác minh trong repo]**: đọc source/config/migration hoặc kết quả graph index của `burp-mcp`.
- **[Nguồn upstream]**: tài liệu hoặc source chính thức của `codebase-memory-mcp`, liên kết trực tiếp tới GitHub.
- **[Khuyến nghị]**: thiết kế đề xuất cho Burp MCP; không phải capability đã tồn tại.

Codebase-memory index của repo tại thời điểm nghiên cứu ở trạng thái ready, full mode; các file Rust/Kotlin được trích dẫn không có recorded coverage gap. Hai SQL migration có parse-partial ranges nên schema SQL được đọc trực tiếp thay vì suy luận chỉ từ graph.

---

## 2. Sitemap graph hiện tại

### 2.1 Luồng hiện tại

```text
MCP client
    │ sitegraph_sync
    ▼
Rust BurpTools
    │ lặp SitemapSnapshot(cursor, limit=500)
    │ không có hard cap tổng; page 500 cho tới end-of-source
    │ lấy ScanIssues page đầu tiên, limit=500
    ▼
Kotlin SitemapFacade
    │ api.siteMap().requestResponses([prefix filter])
    │ materialize toàn bộ List
    │ drop(offset).take(limit)
    ▼
Montoya SiteMap

Rust SyncBatch
    │ một transaction SQLite
    ▼
SqliteGraph
    ├── nodes + node_search FTS5
    ├── edges + evidence
    └── graph_metadata.last_synced_at
```

**Bằng chứng local:**

- `crates/burp-tools/src/lib.rs:1937-2009` đăng ký `sitegraph_sync`, phân trang Rust với page 500, dừng khi hết page hoặc `sitemap.len() >= 10_000`, rồi gọi `scan_issues` một lần với page 500.
- `src/main/kotlin/io/github/nguyenthdat/burpmcp/SitemapFacade.kt:33-61` gọi `requestResponses()`, lấy `entries.size`, sau đó `drop(offset).take(limit)`.
- `src/main/kotlin/io/github/nguyenthdat/burpmcp/rpc/BurpRpcServer.kt:521-568` áp giới hạn gRPC page/serialized response và dùng cursor thực chất là offset.
- `proto/burp.proto:125-142` định nghĩa `SitemapSnapshotRequest/Response` và `SitemapEntry`, gồm URL, method, status, content type, response body, redirect, links, forms, scripts.

### 2.2 Model và persistence đang có

Node kinds hiện tại: `Origin`, `Endpoint`, `PathSegment`, `Parameter`, `ResponseFingerprint`, `Technology`, `Issue`, `Artifact` — [`crates/sitegraph/src/model/node.rs:6-15`](../crates/sitegraph/src/model/node.rs).

Edge kinds hiện tại: `Contains`, `PathChild`, `AcceptsParameter`, `RespondedWith`, `LinksTo`, `FormSubmitsTo`, `LoadsScript`, `RedirectsTo`, `HasIssue`, `HasTechnology`, `HasArtifact` — [`crates/sitegraph/src/model/edge.rs:6-18`](../crates/sitegraph/src/model/edge.rs).

SQLite schema có:

- `nodes(id, kind, stable_hash, created_at, updated_at, metadata)`;
- `evidence(id, source, observed_at, summary)`;
- `edges(id, from_id, to_id, kind, evidence_id, created_at, metadata)`;
- `graph_metadata`;
- FTS5 `node_search` cho kind/origin/method/path/name.

Nguồn: [`crates/sitegraph/migrations/0001_graph.sql`](../crates/sitegraph/migrations/0001_graph.sql), [`crates/sitegraph/migrations/0002_fts.sql`](../crates/sitegraph/migrations/0002_fts.sql), [`crates/sitegraph/src/storage/nodes.rs:5-48`](../crates/sitegraph/src/storage/nodes.rs), [`crates/sitegraph/src/storage/edges.rs:6-38`](../crates/sitegraph/src/storage/edges.rs).

`SqliteGraph::open` tạo parent directory, bật SQLite WAL/foreign keys và chạy migrations — [`crates/sitegraph/src/storage/sqlite.rs:18-29`](../crates/sitegraph/src/storage/sqlite.rs). `sync` gom toàn bộ batch trong một transaction và upsert các node/edge — [`crates/sitegraph/src/storage/sqlite.rs:36-448`](../crates/sitegraph/src/storage/sqlite.rs).

### 2.3 Những điểm đã làm đúng

- Endpoint identity dùng BLAKE3 từ origin + method + normalized path; stable ID helper nằm ở [`crates/sitegraph/src/normalize/fingerprint.rs:1-9`](../crates/sitegraph/src/normalize/fingerprint.rs).
- URL hiện normalize và loại query values khỏi topology metadata; đây là topology identity behavior, không phải policy áp dụng cho exact security-testing evidence. Enrichment findings phải giữ query/body/header/cookie values khi rule match — [`crates/sitegraph/src/normalize/url.rs:10-53`](../crates/sitegraph/src/normalize/url.rs), [`crates/sitegraph/src/normalize/url.rs:56-66`](../crates/sitegraph/src/normalize/url.rs).
- Response fingerprint hiện được tính transient và không lưu raw body trong topology graph; security-testing enrichment target sẽ lưu exact evidence blobs/captures riêng, không nhầm hai storage policies — [`crates/sitegraph/src/normalize/fingerprint.rs:11-13`](../crates/sitegraph/src/normalize/fingerprint.rs), [`crates/sitegraph/src/storage/sqlite.rs:723-747`](../crates/sitegraph/src/storage/sqlite.rs).
- Search, neighbors, trace và export có limit; trace depth bị chặn ở 8 và result ở 500 — [`crates/sitegraph/src/graph/traversal.rs:3-4`](../crates/sitegraph/src/graph/traversal.rs), [`crates/sitegraph/src/storage/sqlite.rs:471-532,569-608,654-686`](../crates/sitegraph/src/storage/sqlite.rs).
- MCP surface đã có `sitegraph_sync`, `sitegraph_search`, `sitegraph_endpoint_detail`, `sitegraph_status`, `sitegraph_stats`, `sitegraph_neighbors`, `sitegraph_trace`, `sitegraph_diff`, `sitegraph_export` — [`crates/burp-tools/src/lib.rs:1937-2160`](../crates/burp-tools/src/lib.rs).
- Offline behavior phù hợp PLAN.md: graph local vẫn query được khi Burp offline; Burp-bound call trả lỗi kết nối — [`PLAN.md:387-396`](../PLAN.md).

### 2.4 Gaps cần sửa trước auto-index

#### A. Evidence/edge identity

`sync` tạo:

```text
sync_id     = BLAKE3("sync", now, sitemap_count)
evidence_id = BLAKE3("evidence", sync_id, "burp_sitemap_snapshot")
edge_id     = BLAKE3(edge_kind, from_id, to_id, evidence_id)
```

Chi tiết: [`crates/sitegraph/src/storage/sqlite.rs:36-48`](../crates/sitegraph/src/storage/sqlite.rs) và [`crates/sitegraph/src/storage/edges.rs:14-27`](../crates/sitegraph/src/storage/edges.rs).

Điều này làm evidence identity thay đổi theo sync timestamp. Test `repeated_sync_is_idempotent...` chỉ so sánh count sau hai lần sync — [`crates/sitegraph/src/storage/sqlite.rs:723-747`](../crates/sitegraph/src/storage/sqlite.rs) — nên chưa chứng minh edge IDs ổn định qua các timestamp khác nhau.

**Khuyến nghị:** edge identity phải là `(from_id, to_id, kind, graph_id)`. Evidence là quan hệ phụ (`edge_evidence`) hoặc bảng observation riêng, có thể có nhiều evidence cho cùng một edge. `evidence_id` không được là thành phần bắt buộc của edge primary key.

#### B. Snapshot không thể biểu diễn deletion

Node upsert chỉ update `updated_at`/metadata và rebuild FTS row — [`crates/sitegraph/src/storage/nodes.rs:11-38`](../crates/sitegraph/src/storage/nodes.rs). Không có `last_seen_run_id`, source ownership, tombstone hoặc reconciliation marker. Vì vậy, entry biến mất khỏi Burp sitemap sẽ tồn tại mãi trong graph.

**Khuyến nghị:** chỉ prune khi snapshot là **complete** cho một scope; prefix sync hoặc page bị truncated không được xóa dữ liệu ngoài page. Đầu tiên đánh dấu `inactive/tombstoned`, chỉ hard-delete ở maintenance command.

#### C. Pagination, memory và yêu cầu không giới hạn tổng history

Kotlin materialize toàn bộ `requestResponses()` trước khi cắt page. Rust hiện giới hạn tổng sitemap ở 10.000 entries và issues chỉ lấy page đầu tiên. Đây là giới hạn correctness trước khi là giới hạn performance.

**Quyết định:** bỏ hoàn toàn hard cap tổng số history được index. Không thay `10.000` bằng một cap lớn hơn. Indexer đọc tuần tự cho đến khi nguồn báo end-of-source; scale được kiểm soát bằng page, transaction, byte, queue và deadline bounds.

- dùng source-level cursor/sequence ổn định thay cho offset nếu Montoya cho phép;
- nếu chỉ có offset, checkpoint phải giữ offset/page token và resume sau restart;
- mỗi page và SQLite transaction vẫn bounded; commit xong mới lấy page tiếp theo;
- snapshot run lưu `complete`, `source_total`, `pages_read`, `items_read`, `last_cursor`, `cancelled` và `error`;
- `complete=true` chỉ khi source trả end-of-source, không bao giờ vì chạm số item cấu hình;
- paginate toàn bộ scanner issues theo cùng nguyên tắc;
- tách `body_limit` khỏi metadata page limit;
- người dùng có thể pause/cancel; lần sau resume hoặc reconcile, không mất checkpoint đã commit;
- status hiển thị progress với `items_indexed`, `source_total` nếu Burp cung cấp và `end_of_source`.

Kotlin vẫn có rủi ro memory vì Montoya trả list. Spike phải đo behavior trên history lớn và ưu tiên event feed để chi phí steady-state không phụ thuộc việc materialize lại toàn history. Đây không phải lý do để cắt bỏ dữ liệu khỏi persistent index.

#### D. Ingestion và enrichment chưa đủ sâu

`crates/sitegraph/src/ingest/html.rs:18-38` dùng một regex giới hạn `a|area|link|form|script`, tối đa 1.024 URL và 8 KiB mỗi URL. `crates/sitegraph/src/ingest/openapi.rs:3-11` trả lỗi cố ý vì OpenAPI ingestion chưa enabled. Trong `SitemapFacade`, `responseLinks`, `formActions`, `scriptSources` hiện được trả về `emptyList()` — [`SitemapFacade.kt:47-59`](../src/main/kotlin/io/github/nguyenthdat/burpmcp/SitemapFacade.kt).

**Quyết định:** enrichment là một pipeline versioned, không phải các regex rời rạc nhét vào sync loop. Mỗi enricher khai báo input surfaces, rule/database version, resource limits, output schema và capture policy. Capture policy ở security-testing profile phải giữ dữ liệu nguyên bản đã match, không redact:

- Retire.js-style JavaScript library/version detection và vulnerability metadata từ một repository snapshot đã pin/verify;
- HaE-style rule packs cho request/response/WebSocket tagging và extraction, với scope, regex engine, capture policy, severity/color/tags và rule provenance;
- HTML/OpenAPI/GraphQL/JS-route/technology enrichers;
- exact token/secret/header/cookie/query/body captures khi rule match, để test parser và kiểm thử security workflow;
- re-enrichment không cần refetch Burp history khi retained evidence đủ dùng.

Hash/fingerprint chỉ dùng cho identity/deduplication. Không được thay exact capture bằng hash trong finding output. Redaction là một profile tùy chọn cho export/chia sẻ, không phải behavior mặc định của enrichment.

Retire.js chính thức mô tả việc nhận diện JavaScript library/version dễ tổn thương, gồm cả thư viện chỉ xuất hiện trong asset thay vì package manifest. HaE Network mô tả multi-engine regex để tag và extract HTTP/WebSocket messages, rule database offline và rule schema cấu hình. Burp MCP nên học data model/provenance của hai hệ này, không nhúng Node runtime hoặc phụ thuộc plugin JVM của chúng.

#### E. Trace/diff/export semantics

Trace recursive CTE chỉ đi theo outgoing edges và không lưu visited set — [`crates/sitegraph/src/storage/sqlite.rs:654-685`](../crates/sitegraph/src/storage/sqlite.rs). `truncated` hiện được suy từ số row trả về bằng limit, `next_cursor` luôn `None`; cycle có thể làm kết quả phình hoặc lặp.

Diff chỉ query nodes `updated_at > since`, phân loại added/updated — [`crates/sitegraph/src/graph/diff.rs:16-53`](../crates/sitegraph/src/graph/diff.rs). Export JSON paginate node nhưng lấy outgoing edge theo từng node; `total` cộng node + edge trong khi `truncated` chỉ dựa node — [`crates/sitegraph/src/export/json.rs:17-77`](../crates/sitegraph/src/export/json.rs).

**Khuyến nghị:** trace có visited-set, `total` đếm đúng trong bounded scope, cursor opaque; diff trả added/updated/removed nodes và added/removed edges; export có snapshot token để node-edge page nhất quán.

---

## 3. Bài học từ codebase-memory-mcp

Đây là những pattern nên học ở cấp kiến trúc, **không phải mã nguồn để copy nguyên xi**. Codebase graph và HTTP sitemap graph có domain khác nhau.

### 3.1 Auto-index và watcher tách thành lifecycle rõ ràng

Upstream README mô tả:

- `auto_index` tự index project mới khi MCP session start;
- project đã index được đăng ký background watcher;
- `auto_watch` điều khiển watcher riêng;
- `auto_index_limit` giới hạn số file được auto-index.

Nguồn: [README — Auto-Index](https://github.com/DeusData/codebase-memory-mcp#auto-index), [Configuration Reference](https://github.com/DeusData/codebase-memory-mcp/blob/main/docs/CONFIGURATION.md#2-cli-managed-runtime-settings).

**Bài học cho Burp:** auto-index, auto-watch và manual sync là ba policy khác nhau. Không nên biến mỗi lần `sitegraph_status` thành một sync ngầm; không nên để một manual sync chạy song song với watcher.

### 3.2 Watcher phải có baseline và retry không mất thay đổi

Upstream watcher theo dõi Git HEAD và dirty-state signature; baseline chỉ commit sau successful reindex. Busy-skip hoặc failed run giữ baseline cũ để retry, tránh “đã thấy thay đổi nhưng chưa index xong” — [upstream `watcher.c`](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/watcher/watcher.c), [raw source](https://raw.githubusercontent.com/DeusData/codebase-memory-mcp/main/src/watcher/watcher.c).

Watcher upstream cũng dùng adaptive poll interval, bounded subprocess deadline/output, cancellation, stale-root handling và deferred free. Đây là pattern phù hợp để chuyển thành:

```text
source_sequence_seen
    ──(graph transaction success)──► source_sequence_committed
    ──(failure/cancel/offline)──────► giữ checkpoint cũ, retry
```

Với Burp, baseline không phải Git SHA mà là `sitemap_generation` hoặc monotonic `event_sequence` từ Kotlin. Khi chưa có event sequence, dùng `snapshot_run_id` + complete flag.

### 3.3 One-writer và project lock

Upstream store công khai contract rằng một store handle không được dùng đồng thời; caller phải serialize hoặc dùng store riêng — [upstream `store.h`](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/store/store.h#L1-L20). Upstream cũng có project mutation leases để serialize các thao tác destructive — [upstream `project_lock.c`](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/daemon/project_lock.c).

**Bài học cho Burp:** SQLite WAL giúp reader/writer coexist nhưng không tự giải quyết logical race. Cần một owner duy nhất cho:

- ingest queue;
- reconciliation/prune;
- migration/backup;
- manual sync và auto sync;
- graph export snapshot.

MVP nên dùng **một `SitegraphIndexer` actor trong Rust process**, không cần daemon cross-session. Nếu sau này nhiều Rust process cùng mở graph, lúc đó mới thêm process lock/lease.

### 3.4 Integrity, backup và publish atomic

Upstream store có API cho transactionally-consistent backup, sealing staging DB, atomic replacement và integrity verdict phân biệt `CORRUPT` với `TRANSIENT` — [upstream `store.h`](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/store/store.h#L135-L310).

**Bài học cho Burp:** graph là cache có giá trị điều tra, không nên xóa DB chỉ vì một lỗi mở tạm thời. Thêm:

- `PRAGMA quick_check` ở startup hiếm/maintenance;
- backup trước migration destructive;
- `.corrupt` quarantine chỉ sau verdict chắc chắn;
- atomic export/import vào staging path;
- không thay graph đang phục vụ query bằng file chưa verify.

### 3.5 Coverage/freshness là dữ liệu sản phẩm

Upstream có `check_index_coverage`, phân biệt no recorded issue với complete guarantee, và README nhấn mạnh graph index là persistent local artifact — [upstream README — Indexing pipeline](https://github.com/DeusData/codebase-memory-mcp#how-it-works), [upstream `check_index_coverage` documentation](https://github.com/DeusData/codebase-memory-mcp/blob/main/docs/llms.txt).

**Bài học cho Burp:** `sitegraph_status` phải trả không chỉ node/edge counts mà còn:

```json
{
  "state": "catching_up",
  "source": "burp_sitemap",
  "project_id": "montoya-project-id",
  "project_name": "Client Assessment",
  "graph_id": "stable-db-id",
  "temporary": false,
  "last_attempt_at": "...",
  "last_success_at": "...",
  "source_cursor": "opaque-next-page",
  "coverage": {
    "complete": false,
    "items_indexed": 250000,
    "end_of_source": false,
    "cancelled": false
  },
  "pending_events": 12,
  "enrichment": {
    "mode": "full",
    "ruleset_versions": {"retirejs": "...", "hae-default": "..."},
    "pending": 40
  },
  "last_error": null
}
```

Không được biểu diễn `last_synced_at` như thể graph luôn đầy đủ nếu chưa đạt end-of-source hoặc Burp offline. History lớn chỉ làm state ở `bootstrapping/catching_up`; không tạo `max_items` truncation.

### 3.6 UI, artifact và semantic search: học sau, không kéo vào MVP

Upstream có built-in graph UI tại localhost và optional compressed team graph artifact — [README — Graph Visualization UI](https://github.com/DeusData/codebase-memory-mcp#graph-visualization-ui), [README — Team-Shared Graph Artifact](https://github.com/DeusData/codebase-memory-mcp#team-shared-graph-artifact). Upstream cũng có BM25, semantic search và nhiều enrichment edges — [README — Search](https://github.com/DeusData/codebase-memory-mcp#search).

Với Burp:

- UI là P2; MCP export + một local read-only HTTP view là đủ để validate trước.
- Team-shared graph artifact không nên mặc định vì sitemap có thể chứa sensitive target metadata.
- Semantic/vector search không nằm trong MVP; endpoint/path/status/method/technology FTS có giá trị cao hơn, rẻ hơn và dễ audit.
- Không nhập các edge kiểu `CALLS`, `LSP`, cross-repo vào sitemap graph trừ khi có use case HTTP rõ ràng.

---

## 4. Target architecture cho auto-index

### 4.1 Thành phần

```text
MCP tools
  ├── sitegraph_sync / status / config / projects
  ├── sitegraph_search / neighbors / trace / diff
  └── sitegraph_enrichment_status / reenrich
                     │ bounded commands
                     ▼
ProjectResolver (Rust + Kotlin project identity)
  ├── persistent Project.id() ──► projects/<stable-hash>.sqlite
  └── temporary project       ──► projects/temp-<random-id>.sqlite
                     │
                     ▼
SitegraphIndexer actor (one writer for active project DB)
  ├── unbounded total history through bounded pages
  ├── bounded mpsc queue and transactions
  ├── source cursor/checkpoint
  ├── retry/backoff/cancellation
  ├── snapshot reconciler
  └── transient extractor + redactor
                     │
                     ├──► bounded enrichment workers
                     │      ├── Retire-style JS/advisory matcher
                     │      ├── HaE-style regex rule packs
                     │      └── HTML/OpenAPI/GraphQL/technology
                     │
                     ▼ typed BurpClient
GrpcActor → Kotlin BurpRpcServer → Montoya Project / SiteMap / HTTP handlers
                     │
                     └── optional bounded event metadata queue

Per-project SQLite graph
  ├── readers: search/neighbors/trace/detail/export
  ├── writer: SitegraphIndexer only
  └── findings: versioned enrichment provenance
```

### 4.2 State machine

```text
                 ┌────────────┐
                 │  disabled  │
                 └─────┬──────┘
                       │ enable/start
                       ▼
                 ┌──────────────┐
                 │ bootstrapping│
                 └─────┬────────┘
             success    │             Burp unavailable
               ▼        │                   ▼
          ┌────────┐    │             ┌─────────┐
          │ ready  │◄───┘             │ offline │
          └──┬─────┘                  └────┬────┘
             │ event/poll                    │ reconnect
             ▼                               │
       ┌──────────────┐                      │
       │ catching_up  │──────────────────────┘
       └──────┬───────┘
              │ queue overflow / partial source / failure
              ▼
        ┌──────────┐ ── successful full snapshot ──► ready
        │ degraded │
        └────┬─────┘
             │ disable/unload
             ▼
        ┌─────────┐
        │ stopped │
        └─────────┘
```

State semantics:

| State | Ý nghĩa | Query behavior |
| --- | --- | --- |
| `disabled` | Auto-index không đăng ký watcher/worker | Query graph cũ bình thường |
| `bootstrapping` | Đang full snapshot đầu tiên | Query graph cũ; status trả progress |
| `ready` | Checkpoint đã commit, coverage biết rõ | Query bình thường |
| `catching_up` | Có event/poll pending | Query trả `stale_by_events` |
| `degraded` | Queue overflow, partial page, parser/storage error | Query được; không claim fresh |
| `offline` | Burp RPC unavailable | Query graph cũ; retry bounded |
| `stopped` | MCP shutdown/extension unload | Không nhận việc mới |

### 4.3 Queue và backpressure

**Khuyến nghị P0:**

- queue bounded theo số event và tổng bytes;
- event mới cùng endpoint được coalesce thành một pending key;
- event producer không chờ SQLite và không gọi network;
- khi queue đầy: drop intermediate duplicate, giữ `RESET_REQUIRED` hoặc `FULL_RECONCILE_REQUIRED` marker;
- không drop checkpoint metadata đã commit;
- auto-index retry exponential backoff có jitter và upper bound;
- một manual sync chỉ enqueue command, không tự mở transaction ngoài actor;
- cancellation dừng giữa các page nhưng chỉ commit checkpoint ở transaction hoàn tất.

Không nên dùng callback Burp để làm graph write. Montoya HTTP handler/proxy handler cần chỉ copy bounded metadata rồi enqueue — [Montoya `HttpHandler` Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/http/handler/HttpHandler.html), [Montoya `Http.registerHttpHandler` Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/http/Http.html), [Montoya extension unload guidance](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/extension/Extension.html).

### 4.4 Snapshot và event feed

#### Giai đoạn 1: snapshot fallback

1. Rust đọc page đầy đủ.
2. Mỗi page normalize/parse rồi ghi transaction bounded.
3. Chỉ sau commit mới cập nhật `source_checkpoint`.
4. Khi `complete=true`, chạy reconciliation cho scope.
5. Khi `truncated=true`, giữ dữ liệu cũ và gắn `coverage.partial`.

#### Giai đoạn 2: typed event feed

Bổ sung protobuf typed, không dùng `invoke(toolName, jsonParams)`:

```proto
service SitemapService {
  rpc Snapshot(SitemapSnapshotRequest) returns (SitemapSnapshotResponse);
  rpc EventsSince(SitemapEventsSinceRequest) returns (SitemapEventsSinceResponse);
}
```

### 4.5 Cấu hình đề xuất

**Default an toàn:** auto-index tắt; graph query cũ vẫn hoạt động.

```text
BURP_MCP_SITEGRAPH_AUTO_INDEX=false
BURP_MCP_SITEGRAPH_MODE=off|startup|watch
BURP_MCP_SITEGRAPH_INTERVAL_SECONDS=30
BURP_MCP_SITEGRAPH_PAGE_SIZE=500
BURP_MCP_SITEGRAPH_QUEUE_CAPACITY=4096
BURP_MCP_SITEGRAPH_BODY_LIMIT=262144
BURP_MCP_SITEGRAPH_ENRICHMENT=off|exact|metadata
BURP_MCP_SITEGRAPH_RULE_PACKS=default,custom
BURP_MCP_SITEGRAPH_RETIRE_DB=embedded|path|off
BURP_MCP_SITEGRAPH_PROJECT_DB_ROOT=<platform-data>/burp-mcp/projects
```

Không có `BURP_MCP_SITEGRAPH_MAX_ITEMS`. Không có CLI/MCP option nào giới hạn tổng số sitemap history. `page_size`, body bytes, queue size, transaction size, timeout và retry delay vẫn phải bounded.

#### Per-project database resolver

Mỗi Burp project có một SQLite file riêng:

```text
<graph-root>/projects/
  <stable-project-id>.sqlite
  <another-project-id>.sqlite
  temp-<random-128-bit-id>.sqlite
```

Kotlin lấy project identity từ Montoya `Project.id()`/`Project.name()` và lưu opaque graph UUID trong `api.persistence().extensionData()`. Montoya mô tả `extensionData()` là storage nằm trong Burp project — [Project Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/project/Project.html), [Persistence Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/persistence/Persistence.html#extensionData()).

- **DB identity:** đọc key extension data như `burp_mcp.sitegraph_db_id`; nếu chưa có thì sinh CSPRNG 128-bit ID, persist key và dùng `projects/<opaque-id>.sqlite`. Vì vậy filename không phụ thuộc project name, hostname hoặc target URL.
- **Persistent Burp project:** extension data đi cùng project nên reopen/reload chọn lại cùng DB; `Project.id()` được lưu làm metadata/cross-check, không dùng raw làm filename.
- **Temporary project:** lần đầu luôn nhận một random graph UUID và file `temp-<random-128-bit-id>.sqlite`; registry đánh dấu `temporary=true` nếu adapter xác định được, nếu không thì dùng session-lifetime marker. Temp DB không được suy ra từ target.
- **Project rename:** DB không đổi tên; metadata cập nhật `project_name`.
- **Project close/unload:** đóng writer/SQLite pool sau khi queue drain hoặc cancellation deadline; DB không bị xóa tự động.
- **Collision/ownership:** file create với exclusive lock; registry lưu `project_id`, opaque `graph_id`, `db_path`, `created_at`, `last_seen_at`, `temporary`, schema version.
- **Cross-project queries:** mặc định không query union. MCP cần chọn project context rõ ràng; federation là P2.

`--graph-path`/`BURP_MCP_GRAPH_PATH` hiện cho phép override một path — [`crates/burp-mcp/src/cli.rs:20-59,89-97`](../crates/burp-mcp/src/cli.rs). Giữ option này cho debug/export, nhưng production resolver phải có project identity và không để một default `default.sqlite` trộn nhiều Burp projects. `graph_id` trong status/query là stable project DB identity, không chỉ là label tự nhập.

Nếu MCP process khởi động trước khi Kotlin trả project identity, dùng trạng thái `awaiting_project`; không mở `default.sqlite` rồi migrate dữ liệu sau. Khi identity xuất hiện, resolve/create DB trước khi bootstrap.

### 4.6 Enrichment pipeline và rule lifecycle

```text
Burp snapshot/event
  ▼
canonicalize identity + preserve exact evidence
  ▼
extract artifacts (HTML/OpenAPI/GraphQL/JS)
  ▼
parallel bounded enrichers
  ├── technology fingerprint
  ├── JavaScript component/version matcher
  ├── vulnerability advisory matcher
  ├── HaE-style regex tag/extract rules
  └── secret/token pattern classifier
  ▼
findings + exact captures + evidence ranges + rule provenance
  ▼
one SQLite transaction per page/batch
```

Mỗi finding phải có `enricher_id`, `enricher_version`, `ruleset_id`, `ruleset_version`, `input_fingerprint`, `confidence`, `severity`, `observed_at`, `source_evidence_id`, `surface`, `byte_start`, `byte_end`, `capture_bytes` và metadata. `capture_bytes` giữ byte-exact match; text/JSON view chỉ là derived representation. Không có `redaction_status` trong core finding schema vì core enrichment không redact.

Với match cần context để xác minh, finding tham chiếu một exact evidence blob hoặc exact request/response/WebSocket message. Blob phải giữ bytes nguyên bản, content type, direction, source entry ID và hash integrity. Có thể deduplicate blob theo content hash, nhưng hash không thay thế content.

Giới hạn áp dụng cho ingest/matching throughput, không thay đổi nội dung match: max payload bytes được chấp nhận, max matches, max capture length và timeout. Nếu input/capture vượt policy, finding phải báo `incomplete=true`/`limit_reason`; không silently truncate rồi giả vờ exact.

`Retire.js` là nguồn tham khảo cho JavaScript library/version và known-vulnerability matching — [Retire.js README](https://github.com/RetireJS/retire.js#readme), [Retire.js repository data](https://github.com/RetireJS/retire.js/tree/master/repository). Không chạy Node/npm trong Rust server. Chuyển database/rule format đã pin vào Rust-compatible snapshot hoặc chạy một offline import/build step trước release; không tải rule database qua network trong lúc xử lý Burp traffic.

`HaE Network` là nguồn tham khảo cho multi-engine regex, tag/extract HTTP/WebSocket messages, offline rules và rule database — [HaE Network README](https://github.com/overspace-labs/HaENet#readme), [rule definitions](https://github.com/overspace-labs/HaENet/blob/main/sources/src/main/resources/rules/Rules.yml). Rule engine Rust phải sandbox bằng regex crate/backtracking-safe policy, match timeout/byte limit, capture limit và không cho arbitrary code execution.

Enrichment modes:

| Mode | Hành vi |
| --- | --- |
| `off` | Chỉ normalize, fingerprint và graph relationships |
| `exact` | Chạy toàn bộ pinned enrichers và persist exact captures/evidence; đây là security-testing default |
| `metadata` | Chỉ technology/parser metadata, dành cho export hoặc graph nhẹ; không phải default kiểm thử |

Rule database update là một operation riêng: verify checksum/signature, build staging index, chạy compatibility/schema check, atomic swap, sau đó enqueue re-enrichment. Re-enrichment phải idempotent và chỉ replace findings thuộc đúng `enricher_id/ruleset_id`; không xóa findings do source khác tạo. Exact evidence blob chỉ được garbage-collect khi không còn observation/finding nào tham chiếu.

## 5. Schema và data model đề xuất

### 5.1 Định danh graph/source

Thêm các bảng:

```sql
CREATE TABLE source_checkpoints (
  graph_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  last_sequence INTEGER,
  last_snapshot_id TEXT,
  last_success_at INTEGER,
  coverage_json TEXT NOT NULL CHECK(json_valid(coverage_json)),
  PRIMARY KEY (graph_id, source, scope)
);

CREATE TABLE sync_runs (
  id TEXT PRIMARY KEY,
  graph_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  started_at INTEGER NOT NULL,
  finished_at INTEGER,
  status TEXT NOT NULL,
  complete INTEGER NOT NULL,
  items_seen INTEGER NOT NULL,
  pages_seen INTEGER NOT NULL,
  error TEXT
);

CREATE TABLE tombstones (
  graph_id TEXT NOT NULL,
  node_id TEXT NOT NULL,
  source TEXT NOT NULL,
  scope TEXT NOT NULL,
  first_missing_at INTEGER NOT NULL,
  last_confirmed_at INTEGER NOT NULL,
  reason TEXT NOT NULL,
  PRIMARY KEY (graph_id, node_id, source, scope)
);

CREATE TABLE project_registry (
  project_id TEXT PRIMARY KEY,
  project_name TEXT,
  db_path TEXT NOT NULL UNIQUE,
  temporary INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  last_seen_at INTEGER NOT NULL,
  schema_version INTEGER NOT NULL
);

CREATE TABLE evidence_blobs (
  id TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL,
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
```

`project_registry` có thể nằm trong một registry DB nhỏ dưới graph root hoặc được quản lý bởi process runtime; từng project graph DB vẫn là file độc lập. `enrichment_findings` phải tách khỏi `issues` do scanner tạo để re-enrichment không overwrite issue lifecycle. `evidence_blobs.payload` và `enrichment_findings.capture` là byte-exact security-testing evidence, không phải metadata provenance placeholder.

Tên/schema cụ thể có thể thay đổi, nhưng các invariant không nên bỏ:

- checkpoint chỉ advance cùng transaction với graph mutation;
- scope và source là bắt buộc để prefix sync không xóa nhầm toàn graph;
- `complete=false` không được kích hoạt deletion;
- run failure không overwrite last successful checkpoint;
- event gap làm invalid checkpoint và yêu cầu reconcile;
- không có `max_items` trong checkpoint hoặc schema; `items_seen` là counter thực tế, không phải quota;
- project identity phải resolve trước khi tạo/open graph DB;
- findings chỉ replace theo `(node_id, enricher_id, ruleset_id, input_fingerprint, byte range)`;
- exact capture và referenced evidence blob phải round-trip byte-for-byte;
- finding vượt resource policy phải khai báo incomplete, không được silently truncate; core exact evidence không redact.

### 5.2 Evidence ổn định

Đề xuất:

```text
node_id = H(graph_id, node_kind, canonical_identity)
edge_id = H(graph_id, from_id, to_id, edge_kind)
observation_id = H(source, source_entry_id, observed_revision)
evidence_id = H(observation_id, extraction_method, extractor_version)
```

Có thể giữ `evidence` hiện tại để tương thích, nhưng thêm `source_event_id`, `source_revision`, `extraction_method`, `extractor_version`, `confidence`, `parse_error`, `first_seen_at`, `last_seen_at`. Evidence không được làm edge identity.

### 5.3 Endpoint và discovered endpoint

Hiện discovered relationship tạo target node với method `GET` mặc định — [`crates/sitegraph/src/storage/sqlite.rs:214-251`](../crates/sitegraph/src/storage/sqlite.rs). Đây là một giả định có thể sai đối với form/API.

Đề xuất:

- method unknown dùng enum/empty sentinel `UNKNOWN`, không biến thành `GET` giả;
- `discovered_by` evidence ghi rõ link/form/script/openapi/js;
- form edge lưu method/action metadata đã redact;
- chỉ chuyển discovered endpoint thành observed endpoint khi Burp thực sự quan sát request.

### 5.4 Seen-set và deletion

Mỗi full snapshot scope phải có `run_id`. Trong transaction:

1. upsert node/edge;
2. ghi `observation_seen(run_id, source_key, node_id)`;
3. sau khi toàn scope complete, mark absent những source-owned records không seen;
4. tạo tombstone trước, hard-delete sau maintenance grace period.

### 5.5 Security-testing evidence retention

PLAN.md ban đầu chọn metadata-only mặc định — [`PLAN.md:666-685`](../PLAN.md), [`PLAN.md:838-861`](../PLAN.md). Yêu cầu cập nhật của sitemap enrichment thay đổi điểm này: **core enrichment phải lưu exact matched data để kiểm thử**, kể cả Authorization, Cookie, query/body values, JWT/API keys hoặc secret-like tokens nếu rule match chúng.

Phân tách dữ liệu:

- **graph topology:** origin, endpoint, relationship và fingerprints cho traversal;
- **exact evidence blobs:** byte-exact request, response, WebSocket message hoặc asset cần để tái hiện finding;
- **exact captures:** byte range và bytes match của từng rule/enricher;
- **derived views:** decoded text/JSON, tags, technology và advisory metadata;
- **integrity metadata:** source entry ID, content hash, timestamps, rule/enricher versions.

Ingest không redact, mask hoặc hash-only exact evidence. Nếu storage/resource policy từ chối payload quá lớn, indexer phải đánh dấu finding incomplete và giữ lý do; không được lưu bản cắt ngắn rồi báo là exact.

Vì mỗi Burp project có DB riêng, exact evidence nằm trong project DB tương ứng. Database và backup phải dùng owner-only permissions; encryption-at-rest có thể bổ sung sau nhưng không được làm thay đổi bytes khi đọc lại. Logs vẫn không dump payload tự động vì log không phải evidence store.

Export/chia sẻ có thể có profile `exact` và `metadata`; `metadata` có thể bỏ blobs/captures. Đây là export projection, không mutate hoặc redact core database.

## 6. MCP/API surface đề xuất

### 6.1 Giữ tool hiện tại, đổi semantics rõ ràng

Không đổi tên các tool hiện tại trong v3. Cập nhật output để mọi tool graph trả:

```json
{
  "graph_id": "default",
  "last_success_at": "...",
  "freshness": "fresh|stale|partial|offline",
  "coverage": {"complete": true, "source": "burp_sitemap"},
  "items": [],
  "total": 0,
  "truncated": false,
  "next_cursor": null
}
```

`last_synced_at` chỉ là field tương thích; không dùng nó như freshness đầy đủ.

### 6.2 Tool/config mới, giới hạn số tool

| Tool | P0/P1 | Mục đích |
| --- | --- | --- |
| `sitegraph_status` | P0 | Trả project/DB identity, state, queue, checkpoint, progress, freshness, ruleset versions, last error |
| `sitegraph_sync` | P0 | Enqueue full/scoped reconcile tới end-of-source; không total cap; serialize với watcher |
| `sitegraph_config` | P0 | Read/set mode, page/resource bounds và enrichment; không có `max_items` |
| `sitegraph_projects` | P0 | Liệt kê project registry và active DB identity; không union-query mặc định |
| `sitegraph_search` | P0 | Query active project DB; thêm scope/kind/method/status/tag/finding filters |
| `sitegraph_neighbors` | P0 | Bounded adjacency, filter edge kind/direction |
| `sitegraph_trace` | P0 | Visited-set, depth/result bound, opaque cursor |
| `sitegraph_diff` | P1 | Added/updated/removed nodes, edges và findings theo sync/event checkpoint |
| `sitegraph_reconcile` | P1 | Explicit full-scope tombstone reconciliation |
| `sitegraph_enrichment_status` | P1 | Ruleset/database versions, re-enrichment progress và failures |
| `sitegraph_reenrich` | P1 | Re-run selected pinned enrichers without refetch khi evidence đủ |
| `sitegraph_export` | P1 | Snapshot-consistent JSON/CSV, optionally GraphML/DOT ngoài MCP |
| `sitegraph_coverage` | P1 | Chi tiết source/page/parser/enricher coverage khi status quá dài |

Không mở `sitegraph_sql`, `sitegraph_cypher` hoặc arbitrary query trong MVP; đúng non-goal PLAN.md: [`PLAN.md:22-29`](../PLAN.md), [`PLAN.md:703-711`](../PLAN.md).

### 6.3 Query correctness

- Cursor phải opaque và gắn với snapshot token khi graph đang mutate.
- Search hỗ trợ node kind, origin, method, status range, content type, technology, issue severity, `active`.
- Trace cần visited node/edge set, hướng đi rõ, total/truncated/next cursor thực sự đúng.
- Diff dùng `sync_run_id` hoặc sequence thay cho timestamp đơn thuần; timestamp có độ phân giải thấp và không biểu diễn ordering tuyệt đối.
- Export node/edge phải cùng một read snapshot, hoặc trả `snapshot_id` để client tải các page nhất quán.

---

## 7. Ingestion roadmap

### P0 — dữ liệu Burp hiện có, không giới hạn tổng history

1. Full sitemap snapshot đọc tới end-of-source; không hard cap tổng item.
2. Scanner issues phân trang đầy đủ tới end-of-source, scope-aware.
3. URL/method/status/content-type/redirect.
4. Parameter names theo location: query/header/cookie/path/body, không values.
5. Response fingerprint transient.
6. HTML links/forms/scripts bounded, parser errors có evidence.
7. Stable source entry key nếu Montoya cung cấp; fallback canonical endpoint + observation revision.
8. Project resolver theo Montoya `Project.id()`; một DB riêng mỗi Burp project; temp DB random name.

### P1 — enrich có giá trị cao

1. OpenAPI JSON/YAML: paths, methods, parameters, schemas/artifact provenance và exact source evidence.
2. GraphQL endpoint/operation names cùng exact query/variables capture khi rule yêu cầu.
3. Technology evidence từ headers/body với exact byte range và confidence.
4. Retire.js-style JS library/version/vulnerability matching bằng database offline đã pin, giữ exact asset evidence.
5. HaE-style regex rule packs cho HTTP/WebSocket tagging/extraction, exact captures, byte ranges và provenance.
6. Secret/token/header/cookie/query/body findings giữ giá trị exact phục vụ kiểm thử.
7. Scanner issue lifecycle: open/resolved/last-seen.
8. Proxy/HTTP handler event feed và sequence checkpoint.
9. Redirect chains, form method, content discovery source.

Enrichment runner phải có `off|exact|metadata` mode; `exact` là security-testing default. Runner chạy idempotent, bounded và không cần refetch khi exact evidence blob đã lưu đủ dùng.

### P2 — phân tích nâng cao

1. Bounded JavaScript route extraction (không execute JavaScript).
2. WebSocket handshake/channel metadata.
3. Endpoint clustering/shortest path/impact analysis với `petgraph` optional; PLAN.md đã để `petgraph` cho in-memory analysis — [`PLAN.md:810-825`](../PLAN.md).
4. Read-only local graph UI.
5. Semantic search/vector embeddings, chỉ bật opt-in sau benchmark và privacy review.
6. Cross-project/federated graph, chỉ khi đã có graph_id/source ownership chắc chắn.

**Trạng thái triển khai 2026-08-24:**

- Hoàn thành bounded JavaScript route extraction, không execute JavaScript; input, route length và match count đều có hard resource bound. Route được persist bằng edge `discovers_route`.
- Hoàn thành WebSocket channel/message topology: channel artifact giữ `web_socket_id` và `upgrade_url`, liên kết message bằng edge `has_message`; exact payload/edited payload tiếp tục nằm trong evidence store.
- Hoàn thành in-memory bounded clustering, directed shortest path và downstream impact analysis; MCP tools là `sitegraph_clusters`, `sitegraph_shortest_path`, `sitegraph_impact`. Implement bằng cấu trúc chuẩn thay vì thêm `petgraph`, tránh dependency khi bài toán hiện tại chỉ cần BFS/grouping.
- Default exact rule pack nằm tại `src/main/resources/sitegraph/default-rules.json`, được Kotlin JAR đóng gói và Gradle kiểm tra checksum; Rust dùng `include_bytes!` cùng file để chỉ có một source of truth khi ship. Đây tốt hơn việc duplicate resource giữa JVM và Cargo.
- Local UI chưa bật: cần local-only bind, CSRF/origin protection và explicit exact-evidence reveal trước khi mở HTTP surface.
- Semantic search chưa bật: chưa có benchmark/privacy review và embedding runtime contract; FTS5 vẫn là default.
- Federation chưa bật: query vẫn project-scoped; cần ADR cho trust, ownership và exact-evidence policy trước cross-project access.
- Sửa blocker report v2: edge upsert conflict theo `(from_id, to_id, kind)` và lookup lại legacy edge ID trước khi ghi `edge_evidence`; regression fixture cover DB migration cũ.
- Sửa report v2 request bridge sang byte-exact protobuf và từ chối duplicate scanner issue theo normalized `(name, URL)`.

---

## 8. Prioritized implementation roadmap

### P0 — Correctness và auto-index foundation

| Hạng mục | Kết quả bắt buộc | Acceptance criteria |
| --- | --- | --- |
| Stable evidence/edge identity | Edge không đổi chỉ vì sync timestamp | Chạy cùng batch ít nhất 10 lần, sleep qua nhiều giây, node/edge/evidence identity ổn định; edge count không tăng |
| Full source pagination without total cap | Không mất sitemap/issues dù history lớn | Fixture có hơn 10.000 sitemap và hơn 500 issues; indexer đọc tới end-of-source; không có `max_items` branch; cancel/resume không mất item |
| One-writer actor | Không race manual/auto | Concurrent `sitegraph_sync` + watcher chỉ có một commit order; không database lock loop/duplicate run |
| Per-project DB resolver | Không trộn Burp projects | Hai project IDs mở hai SQLite files; rename giữ file; temp project tạo random filename; query context không đọc nhầm DB |
| Auto-index config | Opt-in, bounded, observable | `off` không spawn watcher; `startup` bootstrap một lần; `watch` retry được; status phản ánh state/queue/error |
| Checkpoint | Không mất thay đổi sau crash/failure | Inject fail sau page N; restart tiếp tục từ checkpoint cũ hoặc full reconcile, không claim success |
| Tombstone/reconcile | Xử lý item bị xóa | Complete full-scope snapshot mark missing; partial/offline snapshot không xóa |
| Exact enrichment evidence | Finding có thể tái hiện | Exact request/response/WebSocket blob và capture round-trip byte-for-byte, gồm token/header/cookie/query/body values; không redact core data |
| Enrichment provenance | Finding có thể cập nhật an toàn | Rerun cùng input/ruleset không duplicate; ruleset version đổi chỉ replace finding owner đó; database tamper/checksum bị từ chối |
| Trace/diff/export hardening | Query trả semantics đúng | Cycle graph không lặp vô hạn; diff có removed; export `exact`/`metadata` profile rõ ràng và pagination cùng snapshot |
| Operational containment | Evidence không thoát ngoài ý muốn | Exact evidence chỉ nằm trong per-project DB; logs không dump payload; metadata export không chứa blob/capture |
| Shutdown | Không còn worker khi stop/unload | MCP shutdown và Kotlin extension unload join/cancel worker trong bounded deadline |

### P1 — Incremental quality

| Hạng mục | Acceptance criteria |
| --- | --- |
| Kotlin `EventsSince` | Sequence monotonic, gap phát hiện được, queue overflow tạo reconcile marker |
| Event coalescing/backpressure | Queue không tăng vô hạn; duplicate endpoint events được gộp; Burp callback không chờ SQLite/network |
| HTML/OpenAPI/GraphQL parser | Corpus fixtures, limit/depth/bytes enforced; incomplete evidence được khai báo rõ |
| Retire-style matcher | Pinned database nhận diện đúng library/version/advisory fixtures; không cần Node/npm hoặc network runtime |
| HaE-style rule packs | Rule schema, regex bounds, exact byte captures/ranges và provenance được kiểm tra bằng fixtures HTTP/WebSocket |
| Freshness/coverage API | Status phân biệt fresh, partial, offline, degraded; mọi query có evidence summary |
| Backup/migration/integrity | Migration backup trước upgrade; corrupt DB quarantine, transient I/O không xóa graph hoặc exact blobs |
| Filtered graph query | Filter kind/origin/method/status/tag/finding/edge direction deterministic và paginated |
| Project ownership | Hai Burp project DB không trộn endpoint, issue, exact evidence hoặc enrichment findings |

### P2 — UX và analysis

| Hạng mục | Acceptance criteria |
| --- | --- |
| Local graph UI | Read-only, bind local-only, explicit reveal exact evidence, pagination |
| Clustering/path analysis | Benchmark trên graph fixture; depth/CPU/memory limits |
| Semantic search | Benchmark so với FTS; opt-in, local-only, no network/API key |
| Export/import artifact | Integrity hash, schema version, explicit `exact` hoặc `metadata` profile |
| Federation | Auth/trust model, graph ownership, exact-evidence policy, conflict semantics và migration đã được ADR hóa |

### Recommended PR sequence

1. **PR A:** stable evidence/edge IDs, source checkpoint tables, status schema.
2. **PR B:** full pagination + sync-run accounting + one-writer `SitegraphIndexer`.
3. **PR C:** opt-in startup/watch auto-index, retry/backoff, cancellation, shutdown.
4. **PR D:** tombstone/reconciliation, corrected diff/trace/export.
5. **PR E:** Kotlin event queue + typed `EventsSince` contract.
6. **PR F:** parser enrichment, backups/migrations, coverage API.
7. **PR G:** optional UI/analysis/semantic search.

---

## 9. Operational và security constraints

### 9.1 Burp lifecycle

PLAN.md yêu cầu không chạy gRPC work trên proxy callback thread và phải đóng gRPC server/jobs/worker pool khi unload — [`PLAN.md:1178-1184`](../PLAN.md). Montoya Javadoc cũng yêu cầu extension terminate background threads/resources khi unload — [`Extension.html:111-118`](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/extension/Extension.html).

Do đó:

- Rust worker không được phụ thuộc vào JVM callback còn sống mà không có cancellation.
- Kotlin phải gửi `project_id`/`project_name` trong handshake trước khi Rust mở graph; project change phải đóng actor/DB cũ rồi resolve DB mới.
- Kotlin event producer phải fail-open cho Burp traffic: nếu queue đầy, ghi một overflow marker và trả callback ngay.
- Extension unload phải làm event source ngừng publish trước, rồi RPC server đóng, rồi Rust reconnect loop nhận offline/stopped.
- Rust không tự retry vô hạn khi Burp offline.
- Temporary DB không bị xóa khi unload; cleanup là retention policy riêng để không mất dữ liệu điều tra ngoài ý muốn.

### 9.2 Resource budgets

Áp limit độc lập cho:

- page size và thời gian/bytes của từng source call;
- response body bytes/item và transient bytes/page;
- HTML references/item và parser depth/tokens;
- regex input bytes, match count, captures và thời gian;
- số enrichment jobs đồng thời;
- event queue count/bytes;
- SQLite transaction size/time;
- graph traversal depth/results;
- export bytes;
- retry rate và worker CPU.

Không áp limit tổng số page/snapshot hoặc tổng số sitemap item/run. Page size 500 không ngăn một body 1 MiB x 500 khỏi tạo payload lớn; ngược lại, history 500.000 items không được truncate chỉ vì lớn. Indexer xử lý nhiều bounded batches cho đến end-of-source hoặc explicit cancellation.

### 9.3 Privacy threat model

Graph cục bộ vẫn là dữ liệu nhạy cảm: hostname nội bộ, endpoint names, issue names, exact headers/cookies/tokens và technology fingerprint có thể tiết lộ target. Vì vậy:

- file/database directory owner-only;
- không bind graph UI ra `0.0.0.0`;
- không log raw request/response hoặc exact captures;
- core per-project DB giữ exact evidence cho security testing;
- metadata export không có raw bodies/captures; exact export phải explicit `profile=exact`;
- graph backups cần explicit path và integrity check;
- query output bounded, nhưng không redact exact result trong project security-testing context;
- raw evidence không bị drop sau extraction nếu còn finding/observation reference; GC theo reference count.

### 9.4 Offline semantics

Khi Burp offline:

- `sitegraph_search`, `neighbors`, `trace`, `detail`, `diff`, `export` query graph cuối cùng;
- `sitegraph_status` trả `offline` + last successful checkpoint;
- `sitegraph_sync` trả actionable connection error hoặc enqueue retry tùy mode;
- không đánh dấu records absent chỉ vì RPC timeout.

Điều này khớp PLAN.md offline behavior — [`PLAN.md:387-396`](../PLAN.md).

---

## 10. Non-goals rõ ràng

Không đưa các mục sau vào auto-index MVP:

- dedicated graph database thay SQLite;
- arbitrary SQL/Cypher qua MCP;
- semantic/vector search;
- full CyberChef hoặc JavaScript execution;
- active crawling/fuzzing tự động chỉ vì auto-index bật;
- lưu request/response bodies, tokens, cookies, JWT, API keys trong **metadata-only export hoặc shared artifact**; exact core evidence trong per-project security-testing DB là mục tiêu bắt buộc, không phải non-goal;
- cross-project federation/team artifact mặc định;
- daemon account-wide trước khi chứng minh một process actor đủ;
- thay đổi Montoya behavior hoặc làm graph writer trong Kotlin;
- JNI hoặc Rust chạy trong Burp JVM.

Auto-index ở đây nghĩa là **tự đồng bộ dữ liệu Burp đã quan sát/được cung cấp qua typed adapter**, không phải tự động tấn công, crawl ngoài scope hay gửi request mới.

---

## 11. Các câu hỏi cần chốt bằng spike trước khi coding

1. Montoya `Project.id()` có ổn định qua reopen/reconnect và project rename không? Khi thiếu identity, lifecycle nào xác định temporary session kết thúc?
2. Montoya `SiteMap` có source entry identity/generation nào ổn định không, hay chỉ trả `List<HttpRequestResponse>`? Nếu không có, event sequence phải do adapter tự cấp.
3. Có thể lấy snapshot theo page mà không materialize toàn bộ list ở Kotlin không? Nếu không, cần document memory behavior và chuyển trọng tâm steady-state sang event feed, nhưng vẫn index hết history bằng nhiều batches.
4. `HttpHandler` có bao phủ đúng các nguồn request/response cần sitemap không, hay cần Proxy handlers riêng?
5. Burp API callbacks có thread-affinity/lifecycle constraint nào cấm đọc body ngoài callback thread?
6. Retire.js database/rule data sẽ được convert và pin vào Rust-compatible format nào? Update verification dùng checksum hay signature nào?
7. HaE rule schema nào được hỗ trợ trong v1: match/tag/extract, WebSocket surface, capture groups, severity và exact capture policy?
8. Body/evidence retention nào được bật mặc định trong `exact` security-testing mode, và resource limit nào chỉ làm finding `incomplete`?
9. Tombstone grace period bao lâu và có user-facing `hide inactive` filter không?

Các câu hỏi này nên được ghi thành ADR/spike result, không để implicit trong implementation.

### Bổ sung nguồn upstream enrichment

- [Retire.js README — library/version vulnerability detection và Burp/ZAP integrations](https://github.com/RetireJS/retire.js#readme)
- [Retire.js repository data](https://github.com/RetireJS/retire.js/tree/master/repository)
- [HaE Network README — multi-engine regex, tag/extract và offline rule database](https://github.com/overspace-labs/HaENet#readme)
- [HaE Network rule definitions](https://github.com/overspace-labs/HaENet/blob/main/sources/src/main/resources/rules/Rules.yml)

---

## 12. Nguồn tham khảo

### Local repository

- [PLAN.md — Sitemap graph model, storage, tools, tests, phases](../PLAN.md#12-sitemap-graph)
- [PLAN.md — Sitegraph implementation stack](../PLAN.md#127-rust-sitegraph-implementation-stack)
- [Rust sitegraph model](../crates/sitegraph/src/model/)
- [Rust SQLite sync/query implementation](../crates/sitegraph/src/storage/sqlite.rs)
- [Rust ingestion and normalization](../crates/sitegraph/src/ingest/), [normalize](../crates/sitegraph/src/normalize/)
- [Rust MCP sitegraph tools](../crates/burp-tools/src/lib.rs#L1937-L2160)
- [Kotlin SitemapFacade](../src/main/kotlin/io/github/nguyenthdat/burpmcp/SitemapFacade.kt)
- [Kotlin gRPC SitemapSnapshot adapter](../src/main/kotlin/io/github/nguyenthdat/burpmcp/rpc/BurpRpcServer.kt#L521-L568)
- [Typed sitemap protobuf](../proto/burp.proto#L125-L142)
- [Graph migrations](../crates/sitegraph/migrations/0001_graph.sql), [FTS migration](../crates/sitegraph/migrations/0002_fts.sql)
- [Montoya `Project.id()`/`name()` Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/project/Project.html)
- [Montoya persistence Javadoc — project extension data and preferences](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/persistence/Persistence.html)
- [Montoya SiteMap Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/sitemap/SiteMap.html)
- [Montoya HTTP handler Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/http/handler/HttpHandler.html)
- [Montoya extension unload Javadoc](../docs/burp-extensions-montoya-api/docs/javadoc/burp/api/montoya/extension/Extension.html)

### Upstream codebase-memory-mcp

- [Repository](https://github.com/DeusData/codebase-memory-mcp)
- [README — Auto-Index](https://github.com/DeusData/codebase-memory-mcp#auto-index)
- [README — Session Coordination Daemon](https://github.com/DeusData/codebase-memory-mcp#session-coordination-daemon)
- [README — Graph Visualization UI](https://github.com/DeusData/codebase-memory-mcp#graph-visualization-ui)
- [README — Team-Shared Graph Artifact](https://github.com/DeusData/codebase-memory-mcp#team-shared-graph-artifact)
- [README — Indexing pipeline and search](https://github.com/DeusData/codebase-memory-mcp#indexing-pipeline)
- [Configuration Reference](https://github.com/DeusData/codebase-memory-mcp/blob/main/docs/CONFIGURATION.md)
- [Watcher implementation](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/watcher/watcher.c)
- [Watcher public contract](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/watcher/watcher.h)
- [Project mutation lock](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/daemon/project_lock.c)
- [SQLite store contract, backup and integrity APIs](https://github.com/DeusData/codebase-memory-mcp/blob/main/src/store/store.h)
- [Upstream MCP/source documentation index](https://github.com/DeusData/codebase-memory-mcp/blob/main/docs/llms.txt)

---

## 13. Final recommendation

Bắt đầu bằng P0, không bắt đầu bằng UI hoặc semantic search. Định nghĩa invariant trước:

```text
một graph writer cho mỗi active project DB
+ checkpoint chỉ commit sau transaction thành công
+ stable edge identity độc lập evidence timestamp
+ complete snapshot mới được reconcile deletion
+ queue bounded và overflow chuyển thành full reconcile
+ offline không tạo tombstone
+ exact enrichment captures/evidence round-trip byte-for-byte
+ hash không thay thế capture; vượt limit phải báo incomplete
+ metadata export không mutate core exact database
+ unload/shutdown luôn cancel và join worker
```

Khi các invariant này có test và status observability, `sitegraph_sync` sẽ trở thành một control plane cho cùng một indexer thay vì một đường code riêng. Auto-index và enrichment phục vụ security testing bằng exact evidence trong per-project DB; containment nằm ở project isolation, owner-only storage, bounded queries và explicit export profile, không nằm ở việc làm mất dữ liệu bằng redaction.
