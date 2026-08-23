# Code Structure Report
<!-- markdownlint-disable MD013 -->

## 1. Kết luận

Cấu trúc workspace hiện tại đã có đúng các vùng trách nhiệm lớn, nhưng các seam
bên trong chưa đủ sâu:

- `burp-mcp` đang là composition root hợp lý.
- `burp-protocol` đúng chỗ để sở hữu loopback gRPC, nhưng generated protobuf vẫn
  rò sang `burp-tools` và binary.
- `burp-tools` đang là god module: MCP schema, validation, orchestration, chuyển
  đổi protobuf, utility và sitegraph cùng nằm trong một file.
- `utility-core` quá mỏng, còn registry và recipe implementation lại nằm ở
  `utility-tools`; tên crate không phản ánh trách nhiệm thật.
- `sitegraph` có mô hình module tốt nhất workspace, nhưng `SqliteGraph` đang gom
  ingestion, normalization, transaction, query và export vào một implementation
  lớn.
- Phía Kotlin đã tách Montoya facades, nhưng toàn bộ gRPC adapter vẫn tập trung
  trong một file và một service lớn.

Đề xuất không thêm crate theo từng feature. Giữ workspace ở **5 crate Rust**
bằng cách:

1. giữ `burp-mcp`;
2. giữ `burp-protocol`, nhưng biến nó thành seam typed thật sự;
3. giữ `burp-tools`, tách module theo nhóm public tool;
4. **gộp `utility-core` + `utility-tools` thành `utility-engine`**;
5. giữ `sitegraph`, tách implementation theo capability nội bộ.

Mục tiêu: dependency DAG một chiều, generated transport types không vượt qua
protocol seam, mỗi module có interface nhỏ và implementation sâu.

---

## 2. Phạm vi và cách đọc

Báo cáo tập trung vào production structure của:

- Rust workspace trong `crates/`;
- protobuf contract trong `proto/`;
- Kotlin extension trong `src/main/kotlin/`;
- các quyết định kiến trúc trong `PLAN.md`, `README.md` và
  `docs/adr/0001-rust-kotlin-grpc-boundary.md`.

Không đánh giá chi tiết Montoya API mirror trong
`docs/burp-extensions-montoya-api/`, build output, `target/`, `.gradle/` hoặc
test artifacts.

Snapshot khi đọc:

- 6 crate Rust;
- 69 RPC trong một `BurpService`;
- 77 MCP handler trong một `BurpTools` implementation;
- 69 utility operation trong một dispatch table;
- các file tập trung lớn nhất: `burp-tools/src/lib.rs` 3,188 dòng,
  `burp-protocol/src/lib.rs` 1,855 dòng, `utility-tools/src/lib.rs` 1,594 dòng,
  `sitegraph/src/storage/sqlite.rs` 1,007 dòng, `BurpRpcServer.kt` 1,688 dòng.

Các số dòng là tín hiệu để tìm coupling, không phải tiêu chí thiết kế độc lập.

---

## 3. Cấu trúc hiện tại

### 3.1 Workspace và dependency DAG thực tế

```text
burp-mcp
├── burp-tools
│   ├── burp-protocol
│   ├── sitegraph
│   └── utility-tools
│       └── utility-core
└── burp-protocol
```

`cargo metadata --no-deps` cho thấy DAG crate hiện tại không có dependency
cycle:

```text
burp-protocol ->
burp-mcp      -> burp-protocol, burp-tools
burp-tools    -> burp-protocol, sitegraph, utility-core, utility-tools
sitegraph     ->
utility-core  ->
utility-tools -> utility-core
```

Đây là nền tốt. Vấn đề chính nằm trong kích thước interface và coupling của từng
crate, không phải cycle giữa crate.

### 3.2 Trách nhiệm hiện tại

| Crate/module       | Trách nhiệm đang có                                                                                      | Nhận xét                                                           |
| ------------------ | -------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| `burp-mcp`         | CLI, cấu hình, stdio server, probe                                                                       | Gần đúng composition root; `main.rs` nhỏ.                          |
| `burp-protocol`    | generated protobuf, tonic client, bounded actor, reconnect, deadline                                     | Seam quan trọng nhưng đang expose `proto`.                         |
| `burp-tools`       | toàn bộ MCP inputs/outputs, router, 77 handlers, protobuf mapping, utility adapter, sitegraph sync/query | Điểm coupling lớn nhất.                                            |
| `utility-core`     | `DataValue`, limits, metadata, recipe runner                                                             | Quá mỏng để biện minh một crate độc lập.                           |
| `utility-tools`    | catalog, search, dispatch và 69 operation                                                                | Thực tế là utility engine, không chỉ “tools”.                      |
| `sitegraph`        | model, normalize, ingest, SQLite persistence, search, traversal, diff, export                            | Phân package đúng hướng; implementation storage còn quá tập trung. |
| Kotlin facades     | typed Montoya operations và state                                                                        | Tách theo capability khá tốt.                                      |
| `BurpRpcServer.kt` | server lifecycle, interceptor, status mapping, 69 RPC adapter methods                                    | Transport adapter đang thành god file.                             |

### 3.3 Các seam đã đúng

Không nên thay các quyết định sau:

1. **MCP stdio ở Rust, Montoya ở Kotlin.** Đây là deployment seam thật.
2. **Protobuf là cross-language contract duy nhất.** Không quay lại JSON hoặc
   stringly `invoke(tool, params)`.
3. **Bounded actor sở hữu tonic client.** Generated client không đi trực tiếp
   vào MCP runtime.
4. **Sitegraph và utility chạy độc lập với Burp.** Offline behavior này có
   leverage thực tế.
5. **`burp-mcp` là composition root.** Binary nên chỉ parse config, xây
   dependency và chạy server/probe.

---

## 4. Các vấn đề cấu trúc chính

### P0 — Generated protobuf vượt qua protocol seam

`burp-protocol` public `proto`, còn `burp-tools` import trực tiếp hàng chục
request/response generated types. `burp-mcp/src/main.rs` và interop test cũng
gọi generated stub/type trực tiếp.

Điều này trái với interface đã ghi trong plan và ADR: generated tonic/prost
types phải nằm bên trong `burp-protocol`; MCP handlers chỉ biết Rust DTOs.

Hệ quả:

- đổi `.proto` buộc MCP adapter đổi dù public MCP contract không đổi;
- protobuf defaults và transport naming rò vào domain mapping;
- `burp-protocol` chưa phải deep module; caller vẫn phải hiểu transport model;
- actor chứa 69 command variant, 69 forwarding method, một match `execute` dài
  và một match `respond_offline` lặp lại cùng catalog RPC.

**Decision:** `burp-protocol` phải expose một typed Rust client interface theo
capability, không expose `proto` cho production callers.

### P0 — `burp-tools` là god module

Một file hiện chứa:

- public JSON Schema input/output;
- validation;
- 77 tool handlers;
- mapping MCP ↔ protobuf;
- common error JSON;
- local HTTP request helpers;
- utility adapter;
- sitegraph sync orchestration;
- contract tests.

`BurpTools` đồng thời sở hữu `BurpClient` và `Arc<SqliteGraph>`, rồi tự gọi
`utility-tools`. Tool router vì vậy là transport adapter, application
orchestration và domain integration trong cùng module.

Hệ quả:

- thay một capability gây merge conflict trong file chung;
- module interface rất rộng và shallow;
- test chủ yếu kiểm tra catalog/schema/helper riêng lẻ thay vì behavior qua
  module interface;
- sitegraph sync nuốt lỗi scan issue bằng `unwrap_or_default`, còn các handler
  khác encode lỗi bằng nhiều shape khác nhau; lỗi không được chuẩn hóa tại một
  seam.

**Decision:** giữ một `McpServer`, nhưng compose router từ các tool modules theo
capability. Mỗi tool module chỉ làm MCP schema + gọi typed dependency + map
`Result` sang `CallToolResult`.

### P1 — Protocol contract và Kotlin adapter quá tập trung

`proto/burp.proto` có một `BurpService` 69 RPC. `BurpRpcServer.kt` có một
`BurpRpcService` nhận khoảng 20 facade và implement toàn bộ RPC.

Kotlin facades đã cung cấp seam theo capability, nhưng protobuf service không đi
theo seam đó dù `PLAN.md` đã đề xuất `ProxyService`, `HttpService`,
`ScannerService`, `SitemapService`, `JobService`.

Hệ quả:

- một thay đổi capability tác động file proto và adapter chung;
- generated client và actor dispatch phình theo tổng số RPC;
- server constructor biết toàn bộ domain;
- ownership/lifecycle của stateful facade khó nhìn, đặc biệt WebSocket, proxy
  rule, payload và job resources.

**Decision:** chia protobuf theo capability nhưng vẫn chạy trên **một gRPC
server, một channel, một port**. Đây là organizational split, không phải
microservice split.

### P1 — Utility crates đặt seam sai

`utility-core` chỉ có data model, giới hạn và một higher-order `run_recipe`;
registry, catalog, search, operation implementations và wrapper recipe lại nằm ở
`utility-tools`.

Deletion test: xóa `utility-core` chỉ chuyển khoảng 94 dòng vào `utility-tools`;
complexity không phân tán sang nhiều caller. Seam hiện tại không mang leverage
đủ lớn.

Ngoài ra catalog metadata và dispatch match được khai báo riêng, nên thêm
operation phải sửa hai nơi. `OperationInfo.input_kind/output_kind` là string
thay vì enum validated. Error type cũng là `String` xuyên suốt.

**Decision:** gộp thành `utility-engine`; dùng một operation descriptor làm
single source of truth cho metadata + executor.

### P1 — `sitegraph` lộ implementation và gom quá nhiều behavior

Điểm tốt: `model`, `normalize`, `ingest`, `graph`, `storage`, `export` đã tách
đúng ngôn ngữ domain.

Điểm chưa tốt:

- `SqliteGraph::sync` chứa normalization, node/edge construction và persistence
  transaction trong một method lớn;
- query types ở `graph`/`export` nhận trực tiếp `SqlitePool`, nên các module này
  là storage query helpers chứ chưa phải domain modules;
- `SqliteGraph::pool()` public làm seam storage bị thủng, dù hiện production
  caller không dùng;
- page limits được clamp lặp lại ở storage, trong khi MCP layer lại validate
  theo nhiều cách khác nhau;
- `openapi::observations` public nhưng luôn trả “not enabled”; đây là surface
  không có implementation hữu dụng;
- `rusqlite` được khai báo trực tiếp nhưng source production chỉ dùng SQLx;
  comment nói nó dùng để link bundled SQLite, làm ownership runtime khó hiểu.

**Decision:** giữ một `sitegraph` crate và một concrete `SiteGraph` interface;
chia storage implementation theo capability, không tạo thêm repository trait khi
chỉ có một production adapter.

### P2 — Composition root và documentation drift

`burp-mcp/src/main.rs` nhìn chung đúng, nhưng:

- binary probe import generated proto trực tiếp;
- comment nói tool implementations ở `tools.rs`, file đó không tồn tại;
- ADR vẫn mô tả spike chưa cutover và NanoHTTPD còn tồn tại, trong khi README
  nói HTTP/JSON path đã bị xóa;
- comment `ProxyFacade` vẫn nói dùng chung cho compatibility HTTP và gRPC
  adapter.

Các drift này làm AI và maintainer chọn sai seam dù code chạy đúng.

**Decision:** sau structural cutover, cập nhật ADR status hoặc thêm ADR mới, xóa
comment cũ và giữ README chỉ mô tả architecture đang chạy.

---

## 5. Cấu trúc đích đề xuất

### 5.1 Cây thư mục

```text
burp-mcp/
├── Cargo.toml
├── proto/
│   └── burp/v1/
│       ├── common.proto
│       ├── system.proto
│       ├── proxy.proto
│       ├── http.proto
│       ├── target.proto
│       ├── scanner.proto
│       ├── intruder.proto
│       ├── jobs.proto
│       ├── collaborator.proto
│       ├── websocket.proto
│       ├── automation.proto
│       └── config.proto
├── crates/
│   ├── burp-mcp/
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs
│   │       ├── app.rs
│   │       └── probe.rs
│   ├── burp-protocol/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── actor.rs
│   │       ├── command.rs
│   │       ├── config.rs
│   │       ├── error.rs
│   │       ├── model/
│   │       │   ├── mod.rs
│   │       │   ├── common.rs
│   │       │   ├── proxy.rs
│   │       │   ├── http.rs
│   │       │   ├── scanner.rs
│   │       │   ├── jobs.rs
│   │       │   └── websocket.rs
│   │       ├── mapping/
│   │       │   ├── mod.rs
│   │       │   └── ...
│   │       └── generated/
│   │           └── mod.rs
│   ├── burp-tools/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs
│   │       ├── error.rs
│   │       ├── pagination.rs
│   │       ├── output.rs
│   │       ├── burp/
│   │       │   ├── mod.rs
│   │       │   ├── proxy.rs
│   │       │   ├── http.rs
│   │       │   ├── target.rs
│   │       │   ├── scanner.rs
│   │       │   ├── intruder.rs
│   │       │   ├── jobs.rs
│   │       │   ├── collaborator.rs
│   │       │   ├── websocket.rs
│   │       │   └── automation.rs
│   │       ├── utility.rs
│   │       └── sitegraph.rs
│   ├── utility-engine/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── value.rs
│   │       ├── error.rs
│   │       ├── limits.rs
│   │       ├── registry.rs
│   │       ├── recipe.rs
│   │       ├── magic.rs
│   │       └── operations/
│   │           ├── mod.rs
│   │           ├── encoding.rs
│   │           ├── text.rs
│   │           ├── bytes.rs
│   │           ├── crypto.rs
│   │           ├── compression.rs
│   │           ├── jwt.rs
│   │           └── http.rs
│   └── sitegraph/
│       ├── migrations/
│       └── src/
│           ├── lib.rs
│           ├── graph.rs
│           ├── error.rs
│           ├── limits.rs
│           ├── model/
│           ├── normalize/
│           ├── ingest/
│           │   ├── mod.rs
│           │   ├── sitemap.rs
│           │   └── html.rs
│           └── sqlite/
│               ├── mod.rs
│               ├── migrate.rs
│               ├── sync.rs
│               ├── node_store.rs
│               ├── edge_store.rs
│               ├── search.rs
│               ├── traversal.rs
│               ├── diff.rs
│               └── export.rs
└── src/main/kotlin/io/github/nguyenthdat/burpmcp/
    ├── BurpMcpExtension.kt
    ├── config/
    ├── lifecycle/
    ├── burp/
    │   ├── proxy/
    │   ├── http/
    │   ├── target/
    │   ├── scanner/
    │   ├── intruder/
    │   ├── jobs/
    │   ├── collaborator/
    │   ├── websocket/
    │   └── automation/
    └── grpc/
        ├── BurpGrpcServer.kt
        ├── RpcDeadlineInterceptor.kt
        ├── RpcStatus.kt
        ├── SystemGrpcService.kt
        ├── ProxyGrpcService.kt
        ├── HttpGrpcService.kt
        ├── TargetGrpcService.kt
        ├── ScannerGrpcService.kt
        ├── IntruderGrpcService.kt
        ├── JobGrpcService.kt
        ├── CollaboratorGrpcService.kt
        ├── WebSocketGrpcService.kt
        └── AutomationGrpcService.kt
```

Tên file có thể điều chỉnh theo idiom thực tế; quan trọng là responsibility và
dependency direction, không phải đạt một số dòng/file cụ thể.

### 5.2 Dependency DAG đích

```text
burp-mcp
└── burp-tools
    ├── burp-protocol
    ├── utility-engine
    └── sitegraph
```

Quy tắc:

- `burp-mcp` không import `burp_protocol::generated` hoặc protobuf DTO.
- `burp-tools` không import protobuf DTO.
- `sitegraph` và `utility-engine` không phụ thuộc `rmcp`, `tonic` hoặc nhau.
- `burp-protocol` không phụ thuộc `rmcp`, `sitegraph` hoặc utility.
- Không tạo `*-core`, `*-types`, `*-api` crate mới chỉ để chứa DTO.

### 5.3 Interface đích theo module

#### `burp-protocol`

Interface caller nên gần dạng:

```rust
#[derive(Clone)]
pub struct BurpClient { /* actor handle */ }

impl BurpClient {
    pub async fn proxy_history(&self, query: ProxyHistoryQuery) -> Result<Page<ProxyEntry>, BurpError>;
    pub async fn send_request(&self, request: HttpRequest) -> Result<HttpExchange, BurpError>;
    pub async fn start_audit(&self, request: AuditRequest) -> Result<Job, BurpError>;
}
```

Caller chỉ học domain DTO, limits và error modes. Protobuf nằm trong
`generated`, mapping nằm trong crate. `connect_client` chỉ nên public dưới
feature/test support nếu interop test cần gọi raw generated client; production
interface không dùng nó.

Không cần tạo trait cho toàn bộ 69 phương thức chỉ để mock. Boring option:

- production dùng concrete `BurpClient`;
- tests cho `burp-tools` dùng một `BurpBackend` nhỏ theo từng capability nếu có
  ít nhất adapter production + fake;
- các seam test-only giữ private hoặc `pub(crate)`, không expose trong public
  crate interface.

#### `burp-tools`

Interface ngoài duy nhất:

```rust
pub struct McpServer { /* routers + dependencies */ }

impl McpServer {
    pub fn new(deps: Dependencies) -> Self;
}

impl rmcp::ServerHandler for McpServer { /* dispatch */ }
```

Các module `burp::proxy`, `burp::scanner`, `utility`, `sitegraph` cung cấp route
fragments và schema types của chính chúng. `rmcp::ToolRouter::merge` hiện hỗ trợ
compose router; không cần giữ một implementation 77 handler.

Error path thống nhất:

```rust
Result<T, ToolError> -> CallToolResult::structured_error(...)
```

Không encode `{"error": ...}` thành `String` rồi parse lại trong
`mark_embedded_error`. Xóa pass-through này sau khi mọi handler trả structured
result trực tiếp.

`sitegraph_sync` nên gọi một application-level synchronizer, ví dụ
`SiteGraphSync::run(prefix)`, thay vì tự paginate Burp, map observations và
write database trong MCP handler.

#### `utility-engine`

Một descriptor phải sở hữu metadata và executor:

```rust
pub struct Operation {
    pub info: OperationInfo,
    execute: fn(DataValue, &serde_json::Value) -> Result<DataValue, UtilityError>,
}
```

Registry map ID → descriptor. `search`, `describe`, `run`, `run_recipe`, `magic`
cùng dùng registry đó. `InputKind`/`OutputKind` là enum; errors là
`UtilityError`, chỉ được render thành MCP error tại `burp-tools::utility`.

Chia `operations/` theo cohesion của implementation, không tạo crate theo
codec/hash/compression.

#### `sitegraph`

Interface ngoài là concrete deep module:

```rust
pub struct SiteGraph { /* SQLite implementation hidden */ }

impl SiteGraph {
    pub async fn open(config: SiteGraphConfig) -> Result<Self, SiteGraphError>;
    pub async fn sync(&self, batch: SyncBatch) -> Result<SyncSummary, SiteGraphError>;
    pub async fn search(&self, query: SearchQuery) -> Result<Page<Endpoint>, SiteGraphError>;
    pub async fn neighbors(&self, query: NeighborQuery) -> Result<Page<Neighbor>, SiteGraphError>;
    pub async fn trace(&self, query: TraceQuery) -> Result<TracePage, SiteGraphError>;
}
```

`SqlitePool` không thuộc interface. `pool()` chỉ có thể là `pub(crate)` hoặc
test helper nội bộ. Page/depth limits nên là validated types hoặc query
constructors, để storage không silently clamp input khác với MCP validation.

`sync.rs` chịu trách nhiệm build/write graph theo batch; `search.rs`,
`traversal.rs`, `export.rs` chịu SQL của capability. Chúng là implementation nội
bộ, không nhận pool qua public function.

#### Kotlin

Mỗi `*GrpcService` là adapter mỏng giữa generated protobuf và một Montoya facade
tương ứng. Server chỉ assemble services và interceptor:

```kotlin
NettyServerBuilder
    .forAddress(loopbackAddress)
    .addService(SystemGrpcService(...))
    .addService(ProxyGrpcService(proxyFacade))
    .addService(HttpGrpcService(httpFacade))
    .addService(JobGrpcService(jobFacade))
```

Stateful facades tiếp tục là resource owners và implement `AutoCloseable`.
`BurpGrpcServer.close()` đóng chúng theo thứ tự rõ ràng. Không đưa Montoya API
trực tiếp vào từng RPC method nếu facade đã tồn tại.

---

## 6. Protobuf split cụ thể

Đề xuất split theo cohesive capability, không phải một file/RPC:

| Proto service         | Capability                                                                               |
| --------------------- | ---------------------------------------------------------------------------------------- |
| `SystemService`       | ping, echo/probe, server/extension info                                                  |
| `ProxyService`        | history, detail, intercept config/rules, annotations, proxy WebSocket history            |
| `HttpService`         | send one/many, Repeater, scope/cookie khi liên quan HTTP target                          |
| `TargetService`       | sitemap snapshot, target info, scope, cookies nếu muốn ownership target-centric          |
| `ScannerService`      | issues, issue detail/add, reports, start crawl/audit                                     |
| `IntruderService`     | send to Intruder, payload processor/generator/list lifecycle, bounded matrix/race checks |
| `JobService`          | status, cancel, result                                                                   |
| `CollaboratorService` | payload generation, poll interactions                                                    |
| `WebSocketService`    | managed connections và history                                                           |
| `AutomationService`   | handlers, session rules, macros, Bambdas, BChecks, config                                |

Chọn đúng một owner cho mỗi RPC. Không tạo service riêng cho action có 1–2
method nếu nó không có lifecycle hoặc vocabulary riêng.

Migration protobuf cần compatibility-safe:

1. thêm service mới và reuse message types trước;
2. Rust chuyển từng capability sang generated client mới;
3. Kotlin đăng ký service cũ + mới trong một khoảng migration nội bộ;
4. khi mọi caller đã chuyển, reserve RPC/message fields bị xóa;
5. xóa monolithic `BurpService` trong cùng major v3 trước khi freeze production
   protocol.

Đây là ngoại lệ tạm thời cho clean cutover vì contract chạy chéo hai ngôn ngữ;
dual registration chỉ tồn tại trong migration branch, không thành compatibility
shim lâu dài.

---

## 7. Migration plan theo rủi ro

### Bước 1 — Tách file, chưa đổi interface

Mục tiêu: giảm mutation hotspot trước khi đổi contract.

- Tách `burp-tools/src/lib.rs` thành `server`, `error`, `pagination`, `burp/*`,
  `utility`, `sitegraph`.
- Tách `BurpRpcServer.kt` thành server/interceptor/status và service adapters
  theo capability, ban đầu vẫn extend cùng generated `BurpServiceImplBase` nếu
  cần.
- Tách `burp-protocol/src/lib.rs` thành actor/config/error/generated/client
  modules, chưa đổi public methods.
- Tách `utility-tools` operation implementations thành `operations/*`, giữ exact
  catalog và behavior.
- Tách `SqliteGraph` query methods và sync implementation thành private files.

Acceptance: public MCP tool names/schema, protobuf wire contract, utility
fixtures và sitegraph behavior không đổi.

### Bước 2 — Đóng protocol seam

- Thêm Rust domain DTOs trong `burp-protocol::model`.
- Chuyển mapping protobuf ↔ DTO vào `burp-protocol`.
- Chuyển từng `burp-tools` module khỏi `burp_protocol::proto`.
- Chuyển probe sang typed probe interface.
- Làm `generated` private; raw client chỉ mở cho interop tests nếu cần.

Acceptance: grep production code ngoài `burp-protocol` không còn
`burp_protocol::proto` hoặc generated tonic types.

### Bước 3 — Gộp utility crates

- Tạo `utility-engine` từ hai crate hiện tại bằng move/rename, không layer thêm
  facade.
- Chuyển `DataValue`, limits, registry, recipe, magic và operations vào crate
  mới.
- Dùng một descriptor cho metadata + executor.
- Chuyển `String` errors sang `UtilityError` có variants hữu ích.
- Migrate `burp-tools` caller, xóa `utility-core` và `utility-tools` trong cùng
  cutover.

Acceptance: workspace còn 5 crate; operation IDs, outputs và resource limits giữ
nguyên.

### Bước 4 — Deepen sitegraph

- Rename public facade thành `SiteGraph`; giữ `SqliteGraph` private nếu tên
  implementation còn cần.
- Đưa SQL helpers vào `sqlite/*`, không public pool.
- Tạo query types chịu trách nhiệm validate cursor/limit/depth.
- Di chuyển orchestration sync khỏi MCP handler vào module riêng; lỗi issues
  fetch không được im lặng biến thành empty list nếu contract yêu cầu đầy đủ
  sync.
- Xóa hoặc để private `openapi` cho đến khi có implementation thật.
- Làm rõ bundled SQLite ownership; nếu SQLx feature đã đủ, bỏ direct `rusqlite`
  dependency.

Acceptance: MCP adapter không biết SQLx; storage không silently sửa invalid
public input.

### Bước 5 — Split protobuf services

- Thêm proto files/services theo bảng ở mục 6.
- Implement Kotlin adapters và Rust generated clients theo capability.
- Actor vẫn sở hữu một channel và một bounded queue; command catalog có thể chia
  enum nội bộ theo capability nhưng giữ backpressure tập trung.
- Migrate integration fixture từng service.
- Xóa monolithic service sau khi parity hoàn tất.

Acceptance: một capability thay đổi không buộc sửa adapter của capability khác;
một server/port/channel vẫn giữ nguyên.

### Bước 6 — Cleanup bắt buộc

- Xóa stale comments về `tools.rs`, compatibility HTTP và NanoHTTPD.
- Cập nhật ADR status hoặc tạo ADR mới cho production cutover + service split.
- Đồng bộ `PLAN.md` với trạng thái implemented hoặc chuyển plan thành historical
  record.
- Giữ contract tests tại external seams: MCP tool catalog/schema, protocol
  interop, utility engine interface, sitegraph interface.
- Xóa tests chỉ khóa layout/private helper sau khi interface tests thay thế.

---

## 8. Những lựa chọn không đề xuất

### Không tạo crate cho từng Burp capability

`burp-proxy-tools`, `burp-scanner-tools`, `burp-websocket-tools`, v.v. sẽ làm
manifest/dependency/release overhead tăng mà không tạo deployment seam mới.
Module trong `burp-tools` và service trong protobuf đã đủ locality.

### Không tạo một “domain DTO crate” dùng chung Rust/Kotlin

Cross-language source of truth là protobuf. Rust DTO thuộc `burp-protocol`;
Kotlin DTO thuộc facade tương ứng. Một crate/module DTO trung gian dùng chung
chỉ thêm mapping layer và shallow interface.

### Không tạo repository trait cho sitegraph ngay

Hiện chỉ có SQLite adapter. “Một adapter” chưa biện minh seam public. Tests nên
mở SQLite temp database thật; nếu sau này có in-memory hoặc remote adapter thực
sự, khi đó mới extract port từ behavior đã biết.

### Không giữ `utility-core` chỉ vì dependency purity

`utility-engine` vẫn là leaf crate. Gộp không làm nó phụ thuộc MCP/Burp/SQL. Nó
chỉ xóa một seam không có leverage.

### Không chia gRPC thành nhiều process/port

Service split là code organization và contract ownership. Nhiều process/port sẽ
làm lifecycle, auth assumptions, reconnect và packaging phức tạp hơn mà không
giải quyết vấn đề hiện tại.

### Không dùng line-count threshold làm rule

File lớn là tín hiệu. Split phải theo capability và invariants. Một
implementation dài nhưng cohesive tốt hơn nhiều file pass-through.

---

## 9. Invariants cần khóa trong quá trình refactor

1. MCP tool names và JSON Schema chỉ đổi khi có migration note rõ ràng.
2. Raw HTTP request/response vẫn là bytes qua gRPC; không lossily decode trong
   protocol layer.
3. Mọi RPC có deadline; actor queue bounded; reconnect vẫn do protocol module sở
   hữu.
4. Kotlin listener chỉ bind `127.0.0.1` và giữ message/concurrency limits.
5. Utility input/output, recipe length và batch size vẫn bounded.
6. Sitegraph không persist body, parameter values hoặc secrets; sync idempotent;
   query deterministic và paginated.
7. Local utility và stored graph query vẫn chạy khi Burp offline.
8. Stateful Kotlin resources đóng khi extension unload.
9. Generated protobuf không xuất hiện trong MCP public types hoặc tool modules.
10. Dependency DAG giữ một chiều như mục 5.2.

---

## 10. Ưu tiên thực thi

| Ưu tiên | Thay đổi                                      | Giá trị                                           | Rủi ro                                   |
| ------- | --------------------------------------------- | ------------------------------------------------- | ---------------------------------------- |
| P0      | Tách `burp-tools` theo capability             | Giảm hotspot và coupling ngay, không đổi contract | Thấp nếu move-only trước                 |
| P0      | Đóng generated protobuf trong `burp-protocol` | Khôi phục seam đã được ADR yêu cầu                | Trung bình; mapping nhiều                |
| P1      | Tách Kotlin gRPC adapters                     | Align với facades, làm lifecycle rõ               | Trung bình                               |
| P1      | Gộp utility crates                            | Xóa shallow seam, registry sâu hơn                | Thấp–trung bình                          |
| P1      | Deepen `sitegraph` facade                     | Giấu SQLx, chuẩn hóa limits/errors                | Trung bình                               |
| P2      | Split protobuf services                       | Locality dài hạn, giảm blast radius               | Cao nhất do cross-language wire contract |
| P2      | Documentation/test cleanup                    | Ngăn drift và lock interface đúng                 | Thấp                                     |

Thứ tự an toàn: **move-only modularization → close Rust protocol seam → merge
utility → deepen sitegraph → split protobuf → cleanup**.

---

## 11. Definition of done cho cấu trúc mới

Cấu trúc refactor được xem là hoàn tất khi:

- workspace có 5 crate và dependency DAG đúng mục 5.2;
- production code ngoài `burp-protocol` không import generated protobuf;
- `burp-tools` compose route modules, không còn một implementation chứa toàn bộ
  handler;
- utility registry có một source of truth cho metadata + executor;
- public sitegraph interface không expose `SqlitePool`;
- Kotlin server compose nhiều capability adapters thay vì một class sở hữu toàn
  bộ RPC;
- interop test, MCP schema/parity fixtures, utility differential fixtures và
  sitegraph privacy/idempotency tests vẫn pass;
- README và ADR mô tả đúng production architecture, không còn comment legacy
  sai.

---

## 12. Tóm tắt quyết định

Cấu trúc hiện tại không cần “nhiều crate hơn”; cần **ít seam giả hơn và các seam
thật sâu hơn**.

- Giữ các deployment/domain seams thật: MCP binary, protocol client, utility
  engine, sitegraph, Kotlin/Montoya.
- Xóa seam shallow `utility-core`/`utility-tools`.
- Dùng module theo capability bên trong `burp-tools`, `burp-protocol`,
  `sitegraph` và Kotlin gRPC adapter.
- Đóng generated transport types trong `burp-protocol`.
- Split protobuf service theo capability nhưng giữ một server/channel/port.

Đây là cấu trúc boring, dễ test qua interface, giảm blast radius và phù hợp trực
tiếp với target architecture đã ghi trong `PLAN.md`.
