# Burp MCP deep smoke test report v2

## Phạm vi

- Target được ủy quyền: `http://127.0.0.1:3000` (OWASP Juice Shop `20.1.1`).
- Burp MCP: `3.0.0-alpha.1`.
- Burp Suite: Professional `2026.7.3-52685`.
- Browser traffic: CloakBrowser MCP `1.12.0`, upstream Playwright MCP `0.0.79`, Chromium `145.0.7632.109.2`.
- Proxy listener: `127.0.0.1:8080`; target đã nằm trong Burp scope; Intercept được giữ ở trạng thái tắt để automation không bị treo.
- Thời điểm test: `2026-08-22`.
- Đây là deep smoke test, không phải full regression toàn bộ MCP surface.

## Kết luận

Burp MCP hoạt động tốt cho việc tạo và đọc traffic thật qua Proxy: Playwright đi qua listener `8080`, HTTP history/filter/detail, sitemap và WebSocket history đều trả dữ liệu nhất quán. `burp_proxy_websocket_history`, lỗi chính ở report cũ, đã được sửa.

Các phần chưa ổn:

1. `burp_crawl` vẫn không phát sinh request; nay báo lỗi rõ thay vì `completed` giả.
2. Active/default `burp_scan` vẫn trả `Currently unsupported.`; passive scan có chạy và đếm request.
3. `burp_register_http_handler` trả `success: true` nhưng cả add-header lẫn replace-text không được áp dụng lên traffic Proxy.
4. Session handling rule CRUD hoạt động, nhưng rule `language=en -> language=fr` không mutate request qua Proxy.
5. `burp_cookie_jar_set` lưu đảo `domain` và `path`, làm domain filter không đọc lại được cookie vừa set.
6. Không có MCP function để tạo/chạy/list macro. Config export xác nhận Burp hỗ trợ macro nhưng danh sách hiện trống.

## Ma trận kết quả

| Nhóm | Function/luồng | Kết quả | Trạng thái |
|---|---|---|---|
| Browser traffic | Playwright qua Proxy `8080` | HTTP `200/401`, API và WebSocket được ghi nhận | PASS |
| Proxy HTTP | `burp_proxy_history`, `burp_proxy_history_filtered`, `burp_proxy_detail` | Tìm đúng năm request marker và đọc đủ raw request/response | PASS |
| WebSocket | `burp_proxy_websocket_history` | Đọc được 15 message, gồm traffic Playwright mới | PASS — đã fix |
| Sitemap | `burp_sitemap` | Có các API marker mới với status đúng | PASS |
| HTTP handler | register/add-header/replace/remove | CRUD báo success nhưng mutation không xuất hiện | FAIL |
| Proxy rule | register pass-through/remove | Request không treo, trả `200`, có trong history | PASS có giới hạn |
| Cookie jar | list/set/filter | Set báo success nhưng domain/path bị đảo | FAIL |
| Session rules | create/list/remove | CRUD nhất quán | PASS |
| Session rule execution | replace cookie trên Proxy request | Cookie vẫn là `language=en` | FAIL |
| Macro | MCP API | Không có tool macro được mount | NOT AVAILABLE |
| Crawl | job lifecycle | Timeout với `crawl issued no requests before timeout` | FAIL chức năng; PASS báo lỗi |
| Passive scan | `mode: passive` | Completed, `request_count: 18` | PASS |
| Active scan | `mode: active` | `Currently unsupported.` | NOT SUPPORTED |

## 1. Traffic thật bằng Playwright qua Burp

Tạo browser context mới với:

```js
proxy: { server: "http://127.0.0.1:8080" }
```

Các request điều hướng ban đầu đều trả `200`:

```text
/?pw_proxy_v2=20260822b
/rest/products/search?q=apple&pw_proxy_v2=20260822b
/rest/admin/application-version?pw_proxy_v2=20260822b
```

Deep traffic tiếp theo:

```text
GET  /?pw_deep_v2=20260822c                                  -> 200
GET  /api/Quantitys/?pw_deep_v2=20260822c                    -> 200
GET  /rest/products/search?q=banana&pw_deep_v2=20260822c     -> 200
GET  /rest/user/whoami?pw_deep_v2=20260822c                  -> 200
POST /rest/user/login?pw_deep_v2=20260822c                   -> 401
```

Playwright cũng quan sát được Socket.IO WebSocket:

```text
ws://127.0.0.1:3000/socket.io/?EIO=4&transport=websocket&sid=J9BUr8KQwsD3QyIpABk3
```

## 2. Proxy HTTP history, filter và detail

Cả hai function sau đều tìm đúng năm request có marker `pw_deep_v2=20260822c`:

```text
burp_proxy_history
burp_proxy_history_filtered
```

Kết quả gồm các index `233`, `287`, `288`, `289`, `290`, status lần lượt khớp traffic Playwright.

`burp_proxy_detail` tại index `289` trả raw request với các header do Playwright tạo:

```http
X-PW-Evaluate: yes
X-PW-V2-Traffic: 20260822c
Cookie: language=en
```

và response:

```http
HTTP/1.1 200 OK
Content-Length: 11

{"user":{}}
```

Kết luận: browser -> Burp listener -> Proxy history -> MCP read path hoạt động end-to-end.

## 3. WebSocket history — lỗi cũ đã fix

`burp_proxy_websocket_history` với `limit: 50` trả `15` message, gồm:

- `CLIENT_TO_SERVER` và `SERVER_TO_CLIENT`.
- Listener port `8080`.
- Các payload Socket.IO base64 như `MnByb2Jl`, `M3Byb2Jl`, `NQ==`.
- Traffic Playwright mới với `websocket_id: 5` và đúng `sid=J9BUr8KQwsD3QyIpABk3`.

Không còn lỗi:

```text
Burp RPC request failed: Application error processing RPC
```

Phân trang đã được kiểm tra riêng ở retest trước: `limit: 5` trả `next_cursor: "5"`, rồi cursor `5` trả năm item còn lại.

## 4. Sitemap

`burp_sitemap` với prefix `http://127.0.0.1:3000/rest` trả các endpoint mới:

```text
/rest/admin/application-version?pw_proxy_v2=20260822b          -> 200
/rest/products/search?q=apple&pw_proxy_v2=20260822b            -> 200
/rest/products/search?q=banana&pw_deep_v2=20260822c            -> 200
```

Kết luận: traffic Playwright qua Proxy được đưa vào Site map.

## 5. HTTP handler — đăng ký thành công nhưng không mutate

### 5.1 Add-header

Đăng ký:

```json
{
  "header_name": "X-Burp-V2-Handler",
  "header_value": "active-20260822"
}
```

Tool trả:

```json
{"message":"HTTP handler registered","success":true}
```

Sau đó Playwright gửi request qua Proxy tới:

```text
/rest/admin/application-version?pw_handler_header_v2=20260822
```

`burp_proxy_detail` index `294` không có header `X-Burp-V2-Handler`.

### 5.2 Replace-text

Đăng ký:

```json
{
  "match": "/rest/admin/application-version?pw_handler_replace_v2=20260822",
  "replace": "/rest/languages?pw_handler_replace_v2=20260822"
}
```

Playwright vẫn nhận body:

```json
{"version":"20.1.1"}
```

Tức request không bị đổi sang `/rest/languages`.

### 5.3 Cleanup

`burp_remove_http_handler` trả `HTTP handlers cleared`. Không để lại handler sau test.

**Đánh giá:** lỗi semantic nghiêm trọng: API xác nhận register thành công nhưng rule không tác động lên traffic quan sát được. Cả traffic từ `burp_send_request` lẫn Playwright Proxy đều không chứng minh được mutation.

## 6. Proxy interception rule

Đã giữ global interception ở `false`, sau đó đăng ký rule an toàn:

```json
{
  "url_contains": "pw_proxy_rule_v2=20260822",
  "intercept": false
}
```

Playwright request qua Proxy trả `200`; Proxy history ghi index `295`. Remove rule cũng trả success.

**Đánh giá:** PASS cho lifecycle và nhánh pass-through. Không bật `intercept: true` trong automation vì request sẽ bị giữ ở Burp UI và MCP hiện không cung cấp function forward/drop intercepted message; đó sẽ là test dễ treo, không tạo thêm bằng chứng hữu ích.

## 7. Cookie jar — phát hiện domain/path bị đảo

Set cookie:

```json
{
  "domain": "127.0.0.1",
  "name": "burp_mcp_v2",
  "value": "cookie-20260822",
  "path": "/",
  "expiration": "2026-08-23T00:00:00Z"
}
```

Tool trả `cookie updated`, nhưng list với filter:

```json
{"domain":"127.0.0.1","limit":50}
```

không trả cookie vừa set.

List không filter cho thấy:

```json
{
  "name": "burp_mcp_v2",
  "value": "cookie-20260822",
  "domain": "/",
  "path": "127.0.0.1"
}
```

Các cookie test khác lặp lại cùng mẫu, kể cả domain `127.0.0.1:3000`.

**Đánh giá:** `burp_cookie_jar_set` nhiều khả năng truyền nhầm thứ tự `domain` và `path` vào Montoya API. Set trả success nhưng tạo cookie malformed; domain filter vì thế trả rỗng.

Cookie test đã được expire sau khi thu bằng chứng.

## 8. Session handling rules

Lifecycle được kiểm tra:

1. `burp_session_remove_rule` -> success.
2. `burp_session_list_rules` -> `[]`.
3. Tạo rule:

```json
{
  "find": "language=en",
  "replace": "language=fr"
}
```

4. List trả đúng rule vừa tạo.
5. Remove trả success.

Để kiểm tra execution, Playwright set cookie `language=en` rồi request qua Proxy:

```text
/rest/admin/application-version?pw_session_rule_v2=20260822
```

`burp_proxy_detail` index `296` vẫn chứa:

```http
Cookie: language=en
```

không phải `language=fr`.

**Đánh giá:** CRUD PASS; rule execution FAIL trên Proxy traffic. Tên function là “session handling rule” nhưng hành vi quan sát được không tương đương Burp Session Handling Rules thực thi trên request.

## 9. Macro support

Không có function macro trong MCP surface hiện tại. Danh sách mounted tool chỉ có session create/list/remove; không có các thao tác tương đương:

```text
macro_create
macro_list
macro_run
macro_remove
attach_macro_to_session_rule
```

`burp_export_config` xác nhận Burp project có cấu hình macro hợp lệ nhưng đang trống:

```json
{
  "project_options": {
    "sessions": {
      "macros": {
        "macros": []
      }
    }
  }
}
```

**Đánh giá:** không thể test macro end-to-end bằng Burp MCP v3.0.0-alpha.1 vì feature chưa được expose. Đây là thiếu capability, không phải RPC crash.

## 10. Crawl — vẫn chưa hoạt động

Regression job `job-8`:

```json
{
  "url": "http://127.0.0.1:3000/?pw_v2_regression=20260822"
}
```

Lifecycle:

```text
queued -> running -> error
```

Cả status và result cuối cùng trả:

```text
crawl issued no requests before timeout
```

Target vẫn truy cập được và trước đó Playwright tạo thành công nhiều request qua chính listener.

**Đánh giá:** chức năng crawl vẫn FAIL. Phần đã được sửa so với report cũ là không còn báo `completed` gây hiểu nhầm khi `request_count: 0`.

## 11. Scanner

### 11.1 Passive

Regression job `job-9`:

```json
{
  "url": "http://127.0.0.1:3000/?pw_v2_regression=20260822",
  "mode": "passive"
}
```

Kết quả:

```json
{
  "operation": "scanner_audit",
  "state": "completed",
  "request_count": 18,
  "total": 0,
  "error": ""
}
```

Passive path đã xử lý request; `total: 0` nghĩa là không trả issue mới trong job này.

### 11.2 Active/default

Regression job `job-10` với `mode: active` trả:

```text
Currently unsupported.
```

Burp capabilities được công bố có `scanner.read`, không có capability tạo active audit.

**Đánh giá:** passive PASS; active/default NOT SUPPORTED. Lỗi cũ vẫn còn ở nhánh active.

## 12. Danh sách bug còn mở sau v2

### Bug A — HTTP handler false success

- Register trả success.
- Add-header và replace-text đều không ảnh hưởng request qua Proxy.
- Remove hoạt động.

### Bug B — Cookie jar setter đảo domain/path

- Input `domain: 127.0.0.1`, `path: /`.
- Output list thành `domain: /`, `path: 127.0.0.1`.
- Domain filter không tìm thấy cookie vừa set.

### Bug C — Session rule không thực thi

- CRUD đúng.
- Request vẫn có `Cookie: language=en`, không được đổi thành `language=fr`.

### Bug D — Crawl không phát sinh request

- Job chạy rồi timeout.
- Error reporting đã tốt hơn report cũ nhưng chức năng chính chưa hoạt động.

### Bug E — Active scan chưa hỗ trợ

- Job được queue nhưng status/result trả `Currently unsupported.`.
- Passive scan vẫn hoạt động.

### Gap F — Không có macro API

- Burp project có macro config.
- MCP không expose create/list/run/remove/attach macro.

## 13. Các state đã cleanup

- Global Proxy Intercept được giữ `false`.
- HTTP handler rules đã clear.
- Proxy interception rules đã clear.
- Session rules do test tạo đã remove.
- Cookie marker được gửi expiration quá khứ sau test; do bug domain/path, record malformed có thể vẫn xuất hiện tùy cách Burp xử lý expiration.

## Kết luận cuối

Burp MCP hiện đáng tin cậy cho read/observe workflow và traffic generation: Playwright -> Burp Proxy -> HTTP/WebSocket history -> sitemap/detail hoạt động tốt. Các API quản trị nâng cao chưa đồng đều: nhiều lifecycle call trả success nhưng execution layer không tác động (`HTTP handler`, session rule), cookie setter ghi sai field, crawl chưa phát request, active scan và macro chưa khả dụng.
