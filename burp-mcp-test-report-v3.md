# Burp MCP deep smoke test report v3

## Phạm vi

- Target được ủy quyền: `http://127.0.0.1:3000` (OWASP Juice Shop `20.1.1`).
- Burp MCP: `3.0.0-alpha.1`.
- Burp Suite: Professional `2026.7.3-52685`.
- Browser traffic: CloakBrowser/Playwright qua Proxy listener `127.0.0.1:8080`.
- Target đã nằm trong scope; global Proxy Intercept giữ `false` để automation không bị treo.
- Thời điểm test: `2026-08-23` theo marker test; Burp runtime log trả timestamp `2026-08-22T17:xxZ` / `2026-08-23T00:xx+07:00`.
- Chỉ chạy trên Juice Shop local. Payload bounded; không dùng credential thật, không gọi Collaborator và không bật active scan.

## Kết luận

So với report v2, các lỗi sau đã được sửa và tái hiện thành công end-to-end:

1. Macro MCP đã được bổ sung; create/list/run/remove đều hoạt động.
2. HTTP handler đã apply được add-header và text replacement trên cả `burp_send_request` lẫn Playwright Proxy traffic.
3. `burp_cookie_jar_set` không còn đảo `domain`/`path`; domain filter đọc lại đúng cookie.
4. Session rule đã mutate request thật qua Proxy.
5. WebSocket history tiếp tục hoạt động ổn định và phân trang đúng.

Lỗi còn lại:

- `burp_crawl` vẫn không tạo request, cuối cùng báo `crawl issued no requests before timeout`.
- Active scan vẫn không hỗ trợ. V3 đã cải thiện fail-fast: `burp_scan` trả ngay `active scan is unsupported`, không tạo job giả.

Các feature sâu đã hoạt động:

- Intruder nhận request với insertion point ở query, header và JSON body.
- Inline fuzzer thay payload thật và phân biệt status/length từng payload.
- Scanner issue list/detail/pagination hoạt động trên 71 issue.
- Macro chạy request thật và trả HTTP `200`.
- Repeater, managed WebSocket, Bambda, BCheck, Decoder, Sitegraph, parallel requests và race comparison đều được smoke-test.

## Ma trận kết quả

| Nhóm | Function/luồng | Bằng chứng chính | Trạng thái |
|---|---|---|---|
| Browser traffic | Playwright qua Proxy | 5 request marker, HTTP `200/401`, WebSocket mới | PASS |
| WebSocket history | history + cursor | 18 message; cursor `15`, limit `3` trả đúng message mới | PASS |
| Crawl | bounded crawl job | `request_count: 0`, rồi timeout | FAIL |
| Passive scanner job | `mode: passive` | completed, `request_count: 18` | PASS |
| Active scanner job | `mode: active` | fail-fast `active scan is unsupported` | NOT SUPPORTED |
| HTTP handler | add-header + replace | index `358` chứa header và query edited | PASS — fixed |
| Cookie jar | set + domain filter | domain/path đúng | PASS — fixed |
| Session rule | create/list/execute/remove | index `357`: `V3ORIGINAL -> V3EDITED` | PASS — fixed |
| Macro | create/list/run/remove | macro trả response Juice Shop `200` | PASS — new/fixed |
| Intruder | query/header/body insertion | hai request được mở thành công | PASS có giới hạn |
| Inline fuzzer | 3 payload | `200/200/500`, 3 response lengths | PASS |
| Scanner issue read | list/detail/pagination | 71 issue, cursor `10 -> 15` | PASS |
| Repeater | raw request + tab name | mở request thành công | PASS |
| Managed WebSocket | create/send/list/close | `ws-1` lifecycle thành công | PASS |
| Bambda import | official `.bambda` format | `LOADED_WITHOUT_ERRORS` | PASS |
| BCheck import | disabled valid v2-beta check | `LOADED_WITHOUT_ERRORS` | PASS |
| Decoder | base64 + recipe | deterministic results | PASS |
| Sitegraph | sync/search/detail/stats | 775 nodes, 849 edges | PASS |
| Parallel HTTP | 3 request | cả ba HTTP `200` | PASS |
| Race comparison | 4 concurrent request | `responses match`, unique length `1` | PASS |

## 1. Traffic Playwright qua Burp

Playwright mở browser context với:

```js
proxy: { server: "http://127.0.0.1:8080" }
```

Traffic marker `pw_v3_proxy=20260823`:

```text
GET  /?pw_v3_proxy=20260823                               -> 200
GET  /rest/products/search?q=apple&pw_v3_proxy=20260823  -> 200
GET  /api/Quantitys/?pw_v3_proxy=20260823                -> 200
GET  /rest/user/whoami?pw_v3_proxy=20260823              -> 200
POST /rest/user/login?pw_v3_proxy=20260823               -> 401
```

Proxy history trả đúng 5 item tại index `299`, `353`–`356`. Playwright quan sát WebSocket:

```text
ws://127.0.0.1:3000/socket.io/?EIO=4&transport=websocket&sid=X1efzG1YiiN9fCZIAFlB
```

## 2. WebSocket history — tiếp tục PASS

`burp_proxy_websocket_history` trả tổng `18` message. Ba message mới của `websocket_id: 6`:

```text
index 15 CLIENT_TO_SERVER payload MnByb2Jl
index 16 SERVER_TO_CLIENT payload M3Byb2Jl
index 17 CLIENT_TO_SERVER payload NQ==
```

Phân trang:

```json
{"cursor":"15","limit":3}
```

trả đúng ba item trên, `truncated: false`.

Managed WebSocket cũng hoạt động:

```text
create -> ws-1
send_text("2probe-v3-edited") -> success
list -> ws-1
close -> success
```

## 3. Crawl — vẫn FAIL

Job `job-1`:

```json
{"url":"http://127.0.0.1:3000/?pw_v3_crawl=20260823"}
```

Trong lúc chạy:

```json
{
  "state": "running",
  "request_count": 0,
  "total": 0,
  "items": []
}
```

Cuối cùng cả status/result đều trả:

```text
crawl issued no requests before timeout
```

Target và listener vẫn hoạt động ở cùng thời điểm. Đây là bug chức năng còn mở; error reporting vẫn đúng như v2.

## 4. Scanner

### Passive scan

Job `job-2`, `mode: passive`:

```json
{
  "state": "completed",
  "request_count": 18,
  "total": 0,
  "error": ""
}
```

Passive scan tiếp tục PASS.

### Active scan

Gọi `mode: active` trả ngay:

```text
active scan is unsupported
```

Không còn lifecycle `queued -> unsupported` như v2. Đây là fail-fast đúng với mô tả tool mới: “Start a Burp passive audit job; active auditing is unsupported”.

### Scanner issues

`burp_scan_issues` trả `total: 71`. Trang đầu chứa các issue như:

- `Unencrypted communications` — LOW/CERTAIN.
- `Cross-domain Referer leakage` — INFORMATION/CERTAIN.
- `OpenAPI definition found (active scan check)`.
- `Private IP addresses disclosed`.
- `Input returned in response (reflected)`.

`burp_scan_issue_detail` đọc đúng index `0` và `4`. Cursor `10`, limit `5` trả index `10`–`14`, `next_cursor: "15"`.

## 5. HTTP handler — fixed

Đăng ký handler:

```json
{
  "header_name": "X-Burp-V3-Handler-Final",
  "header_value": "proxy-applied",
  "match": "handler-final-original",
  "replace": "handler-final-edited"
}
```

Playwright gửi qua Proxy:

```text
/rest/products/search?q=handler-final-original&pw_v3_handler_proxy=20260823
```

Proxy history index `358` ghi URL đã đổi:

```text
/rest/products/search?q=handler-final-edited&pw_v3_handler_proxy=20260823
```

Raw request chứa:

```http
X-Burp-V3-Handler-Final: proxy-applied
```

`burp_send_request` cũng trả raw request với query `V3EDITED` và header `X-Burp-V3-Handler: applied-20260823`.

Kết luận: lỗi false-success ở v2 đã fix.

## 6. Cookie jar — fixed

Set:

```json
{
  "domain": "127.0.0.1",
  "name": "burp_mcp_v3_cookie",
  "value": "cookie-v3-20260823",
  "path": "/",
  "expiration": "2026-08-24T00:00:00Z"
}
```

Domain filter trả đúng:

```json
{
  "name": "burp_mcp_v3_cookie",
  "value": "cookie-v3-20260823",
  "domain": "127.0.0.1",
  "path": "/"
}
```

V2 records malformed cũ (`domain: "/"`, `path: "127.0.0.1"`) vẫn còn trong jar nhưng record v3 mới được tạo đúng. Kết luận: setter bug đã fix; old malformed data không tự migrate.

## 7. Session rule — fixed

Tạo rule:

```json
{
  "find": "X-V3-Original: V3ORIGINAL",
  "replace": "X-V3-Original: V3EDITED"
}
```

Playwright gửi header `X-V3-Original: V3ORIGINAL` qua listener. Proxy detail index `357` ghi:

```http
X-V3-Original: V3EDITED
```

Create/list/remove hoạt động. Kết luận: execution bug v2 đã fix.

Lưu ý: `burp_send_request` response không giữ custom input header `X-V3-Original`; do đó bằng chứng execution dùng Playwright Proxy traffic, nơi raw request quan sát được đầy đủ.

## 8. Macro MCP — feature mới hoạt động end-to-end

Tạo macro `burp-mcp-v3-macro-20260823` với request:

```http
GET /rest/admin/application-version?pw_v3_macro=20260823 HTTP/1.1
Host: 127.0.0.1:3000
Connection: close
```

Kết quả:

- `burp_macro_create` -> success, serial `39292490054083`.
- `burp_macro_list` -> trả đúng description, item, raw request và serial.
- `burp_macro_run` -> response HTTP `200`, body `{"version":"20.1.1"}`.
- `burp_macro_remove` -> success.
- List sau remove -> `macros: []`.

Kết luận: gap macro của v2 đã đóng.

## 9. Intruder và insertion points

Mở hai request trong Intruder:

### GET — query và header

```http
GET /rest/products/search?q=§V3ORIGINAL§&pw_v3_intruder=20260823 HTTP/1.1
Host: 127.0.0.1:3000
X-V3-Insertion: §HEADERORIGINAL§
```

### POST — JSON body

```http
POST /rest/user/login?pw_v3_intruder=20260823 HTTP/1.1
Host: 127.0.0.1:3000
Content-Type: application/json

{"email":"§v3-user@example.invalid§","password":"§v3-password§"}
```

Cả hai trả `request opened in Intruder`, success. Tên tab riêng cũng được chấp nhận.

Giới hạn: API này chỉ mở request trong UI; không có function list/read Intruder tabs hoặc khởi chạy Burp Intruder attack để xác minh UI state qua RPC. Payload execution được kiểm tra bằng inline fuzzer bên dưới.

## 10. Payload replacement / inline fuzzer

Template:

```http
GET /rest/products/search?q=§PAYLOAD§&pw_v3_fuzzer=20260823 HTTP/1.1
Host: 127.0.0.1:3000
X-V3-Payload: §PAYLOAD§
```

Wordlist:

```text
apple
banana
V3-EDITED-'"<>
```

Job `job-3` completed với ba kết quả:

| Payload | Status | Response length |
|---|---:|---:|
| `apple` | 200 | 921 |
| `banana` | 200 | 277 |
| `V3-EDITED-'"<>` | 500 | 1108 |

`total: 3`, `unique_lengths: 3`, `verdict: completed`.

Đây là bằng chứng payload marker được thay thật, không chỉ mở UI. Payload đặc biệt tạo HTTP `500`; đây là hành vi Juice Shop quan sát được, không phải RPC failure.

## 11. Repeater

`burp_send_to_repeater` mở raw GET request với tab `MCP V3 baseline` và trả success. Theo contract, function chỉ display request, không gửi request. Không có read-back Repeater tab API.

## 12. Bambda và BCheck

### Bambda

Format `.bambda` hợp lệ lấy theo format chính thức PortSwigger:

```yaml
id: 5bb3450f-99f4-4c85-8a8a-0abca18c7e23
name: Burp MCP V3 marker filter
function: VIEW_FILTER
location: PROXY_HTTP_HISTORY
source: |+
  return requestResponse.request().url().contains("pw_v3");
```

Import trả:

```json
{"errors":[],"status":"LOADED_WITHOUT_ERRORS","success":true}
```

Các lần thử trước với raw Java hoặc JSON trả validation errors (`id/name/function/location/source required`). Đây là expected input validation, không phải function bug.

### BCheck

Import disabled BCheck v2-beta hợp lệ:

```text
metadata:
  language: v2-beta
  name: "Burp MCP V3 marker check"
  description: "Disabled smoke-test BCheck"
  author: "burp-mcp-test"
  tags: "mcp", "smoke"

given response then
  if {latest.response} matches "PW_V3_NEVER_MATCH_20260823" then
    report issue:
      severity: info
      confidence: certain
      detail: "Unreachable smoke marker"
  end if
```

Import trả `LOADED_WITHOUT_ERRORS`. Check được import với `enabled: false`, nên không chạy scanner logic và không tạo issue.

Lưu ý cleanup: MCP chưa expose function remove Bambda/BCheck. Hai script smoke-test hợp lệ còn trong Burp library; report ghi rõ để xóa thủ công nếu cần.

## 13. Decoder

Single operation:

```text
base64.encode("V3 payload") -> "VjMgcGF5bG9hZA=="
```

Recipe encode/decode trên `V3 payload: apple/banana` hoàn tất deterministic. Recipe result cuối ở dạng bytes, base64:

```text
VjMgcGF5bG9hZDogYXBwbGUvYmFuYW5h
```

## 14. Sitegraph

Sync prefix Juice Shop:

```text
upserted_nodes: 3355
upserted_edges: 2890
total_nodes: 775
total_edges: 849
```

Search `application-version` trả endpoint stable ID:

```text
5b9b889bd2eb7925753e4dc1b79954356f951951f33e109bfadd0d63fb78eede
```

Detail trả:

```text
origin: http://127.0.0.1:3000
method: GET
path: /rest/admin/application-version
```

Sitegraph chỉ lưu metadata; không lưu request/response body hoặc parameter values.

## 15. Parallel requests và race comparison

### Parallel HTTP

Gửi ba request đồng thời:

```text
/rest/admin/application-version?pw_v3_parallel=1 -> 200
/rest/admin/application-version?pw_v3_parallel=2 -> 200
/rest/products/search?q=banana&pw_v3_parallel=3  -> 200
```

Cả ba trả full raw request/response.

### Race comparison

Job `job-4`, count `4`, request GET application-version:

```json
{
  "state": "completed",
  "total": 4,
  "error_count": 0,
  "unique_lengths": 1,
  "verdict": "responses match"
}
```

Bốn result đều status `200`, length `20`.

## 16. Bug/gap còn mở sau v3

### Bug A — Crawl không phát request

- Job chạy nhưng `request_count` luôn `0`.
- Kết thúc bằng timeout error.
- Đây là bug chức năng còn lại từ v2.

### Gap B — Active scan không hỗ trợ

- V3 fail-fast rõ ràng, không còn queued job gây nhầm.
- Passive scan và scanner issue reading hoạt động.

### Gap C — Không có read-back Intruder/Repeater UI state

- Send-to functions trả success.
- RPC không cung cấp list/detail tab hoặc attack configuration read-back.
- Inline fuzzer là đường executable để xác minh payload replacement.

### Gap D — Không có remove Bambda/BCheck

- Import hoạt động và validate cú pháp.
- Hai valid disabled/import-only smoke scripts không thể cleanup qua MCP hiện tại.

### Observation E — Job `request_count` không nhất quán

- Inline fuzzer và race job trả đầy đủ `items`/`total`, nhưng `request_count: 0`.
- Đây có thể là field chỉ dùng cho crawl/scanner hoặc semantic chưa thống nhất. Không ảnh hưởng execution vì item/status/length chứng minh request đã chạy.

## 17. Cleanup

Đã cleanup hoặc xác minh:

- Macro temporary: removed; list rỗng.
- HTTP handlers: clear sau thu evidence.
- Session rule: remove sau thu evidence.
- Proxy rules: clear.
- Managed WebSocket: closed.
- Global Proxy Intercept: giữ `false`.
- Cookie `burp_mcp_v3_cookie`: expire bằng expiration quá khứ sau test.
- Crawl job đã kết thúc lỗi; không còn background job cần cancel.

Còn lại do thiếu remove API:

- Bambda `Burp MCP V3 marker filter`.
- Disabled BCheck `Burp MCP V3 marker check`.

## Kết luận cuối

Burp MCP v3 retest đã sửa phần lớn execution bugs của v2: handler, cookie setter, session rule và macro đều hoạt động end-to-end. Read/observe paths, payload fuzzing, scanner issue access, WebSocket, sitegraph và concurrency ổn định. Vấn đề chính còn lại là crawl không thực hiện request; active scan là capability chưa hỗ trợ chứ không còn lifecycle lỗi. Intruder/Repeater hiện phù hợp cho “open in UI”, còn automated payload execution nên dùng inline fuzzer/race/parallel request tools.
