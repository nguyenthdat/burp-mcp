# Sitegraph Reference

Sitegraph is an advanced, manual opt-in capability in Burp MCP. It is a local SQLite metadata graph owned by the native Rust server. It is separate from Burp's built-in **Target > Site map** and separate from a target's `/sitemap.xml` resource.

---

## 1. Enable Explicitly

Sitegraph is disabled by default in v3. Enable it only when the target, retention policy, and local graph location are understood:

```json
{
  "command": "/absolute/path/to/burp-mcp",
  "args": [
    "serve",
    "--enable-sitegraph",
    "--sitegraph-mode",
    "off"
  ]
}
```

Environment variable equivalent:

```sh
BURP_MCP_ENABLE_SITEGRAPH=true burp-mcp serve
```

### Configuration Options

| Setting / CLI Flag | Environment Variable | Default | Meaning |
|---|---|---|---|
| `--enable-sitegraph` | `BURP_MCP_ENABLE_SITEGRAPH` | `false` | Expose the 15 `sitegraph_*` MCP tools and initialize the local graph. |
| `--sitegraph-project-root <PATH>` | `BURP_MCP_SITEGRAPH_PROJECT_ROOT` | Platform data directory | Parent directory for project-scoped SQLite databases. |
| `--sitegraph-mode <MODE>` | `BURP_MCP_SITEGRAPH_MODE` | `off` | Sync mode: `off`, `startup`, or `watch`. |
| `--sitegraph-interval-seconds <SECS>` | `BURP_MCP_SITEGRAPH_INTERVAL_SECONDS` | `30` | Delay between bounded `watch` sync attempts. |
| `--sitegraph-rules-path <PATH>` | `BURP_MCP_SITEGRAPH_RULES` | `~/.config/burp-mcp/default-rules.toml` | Sitegraph enrichment rules file (typed TOML format). |

*Note: Merely specifying `--sitegraph-project-root` or `--sitegraph-mode` does not enable sitegraph without `--enable-sitegraph`.*

### 1.1 Enrichment Rules (Typed TOML)

Enrichment rules are defined in declarative TOML files (`.toml`). The embedded `2026.09.05` pack contains 105 rules: 28 original rules plus 77 bounded detectors for cloud credentials, authentication, frameworks, disclosures, infrastructure, vulnerability-prone parameters, PII, and security misconfigurations.

The schema uses a `[pack]` table for metadata and `[[rules]]` array-of-tables for individual rule detectors. TOML literal strings (`'...'`) allow regex patterns without escape complications. Typed deserialization strictly denies unknown fields at document, pack, and rule levels. Supported fields include capture group indexes, severity ratings (`critical`, `high`, `medium`, `low`), and surface filters (`request_message`, `response_message`, `response_body`, `websocket_payload`, `websocket_edited_payload`). All 105 rules are ordinary regex entries without engine condition systems or absence inference; `missing_content_type_options` detects only explicitly insecure header values (`none|0|false|off`). Matching uses surface-partitioned byte-safe `RegexSet` instances, preserving exact present captures in finding metadata.

Example `default-rules.toml` snippet:

```toml
[pack]
id = "burp-mcp-sitegraph"
version = "2026.09.05"
max_matches = 512

[[rules]]
id = "jwt"
pattern = 'eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}'
capture_group = 0
severity = "high"
surfaces = [
    "request_message",
    "response_message",
    "response_body",
    "websocket_payload",
    "websocket_edited_payload",
]

[[rules]]
id = "cors_wildcard_credentials"
pattern = '(?im)(?:^access-control-allow-origin:\s*\*\s*$[\s\S]{1,512}?^access-control-allow-credentials:\s*true\b|^access-control-allow-credentials:\s*true\b[\s\S]{1,512}?^access-control-allow-origin:\s*\*\s*$)'
capture_group = 0
severity = "high"
surfaces = ["response_message"]

[[rules]]
id = "missing_content_type_options"
pattern = '(?im)^x-content-type-options:\s*(?:none|0|false|off)\b'
capture_group = 0
severity = "low"
surfaces = ["response_message"]
```

Dual-format compatibility is not supported: old `.rules` DSL and legacy JSON files fail to load. Unknown fields at the document, pack, or rule level are rejected.

---

## 2. Data and Privacy Boundary

The graph stores normalized endpoint metadata, parameter names, relationships, observations, evidence summaries, bounded provenance, and project-local exact HTTP/WebSocket evidence.

- **Normalized metadata excludes parameter values**: Parameter values and credentials are not copied into normalized nodes.
- **Exact evidence is sensitive**: Indexed raw requests, responses, and WebSocket payloads can exist in the SQLite evidence store and `profile=exact` exports.
- **File Permissions & Retention**: Treat the SQLite database as sensitive engagement data. Apply local filesystem permissions (`chmod 600`), define retention, redact exports, and remove it when the assessment engagement ends.
- **Project Partitioning**: Partitioned by Burp project identity. Do not share a fallback database across unrelated assessments.

---

## 3. Operating Modes

- **`off`** (Recommended): No automatic background sync. Manual `sitegraph_sync` tool calls remain available.
- **`startup`**: Performs one bounded sync after server startup.
- **`watch`**: Periodically performs bounded syncs until shutdown. Sync failures are logged and do not silently fail other tools.

---

## 4. Tool Groups (15 Tools)

| Tool | Parameters | Purpose |
|---|---|---|
| `sitegraph_sync` | `{url_prefix?}` | Synchronize bounded Burp observations into the active project's local SQLite graph. Automatically annotates interesting newly indexed HTTP Proxy history entries matching medium+ severity rules (stable ID lookup, capped at 50/run, critical/high -> RED, medium -> ORANGE, low -> YELLOW; preserves existing notes and non-NONE highlights; idempotent `[SiteGraph]` marker; non-fatal post-commit; HTTP only, never WebSockets). |
| `sitegraph_status` | `{}` | Read local sitegraph synchronization and schema status. |
| `sitegraph_stats` | `{}` | Return graph ID, active mode, node count, and edge count. |
| `sitegraph_config` | `{}` | Read active auto-index settings; edit configuration and restart to change them. |
| `sitegraph_projects` | `{}` | List the active project-scoped graph identity. |
| `sitegraph_search` | `{query, limit?, cursor?}` | Search normalized endpoints with metadata queries. |
| `sitegraph_history_search` | `{query, source?, limit?, cursor?}` | Search indexed raw HTTP/WebSocket evidence with bounded pagination. |
| `sitegraph_endpoint_detail` | `{id}` | Get full normalized endpoint metadata and adjacency counts. |
| `sitegraph_neighbors` | `{id, limit?, cursor?, direction?, edge_type?}` | Page adjacent inbound and outbound graph nodes. |
| `sitegraph_trace` | `{id, max_depth?, limit?, direction?, edge_type?}` | Trace graph relationships to a depth of 1..8 hops. |
| `sitegraph_shortest_path` | `{from_id, to_id, max_depth?}` | Find the shortest directed path between two graph nodes. |
| `sitegraph_clusters` | `{limit?}` | Cluster project endpoints by origin and path segments. |
| `sitegraph_impact` | `{id, max_depth?, limit?}` | Perform downstream impact analysis from a seed node. |
| `sitegraph_diff` | `{since, limit?, cursor?}` | Query nodes changed since a specific Unix timestamp. |
| `sitegraph_export` | `{profile?, format?, snapshot_id?, cursor?, limit?}` | Export bounded metadata or exact-evidence pages; exact evidence is sensitive. |

---

## 5. Recommended Workflow

1. Confirm authorization and the target boundary.
2. Set a narrow Burp target scope before collecting traffic.
3. Start the MCP server with `--enable-sitegraph --sitegraph-mode off`.
4. Review Burp HTTP history and **Target > Site map** first.
5. Run a bounded `sitegraph_sync` with a specific URL prefix.
6. Inspect `sitegraph_status` and `sitegraph_stats`.
7. Use `sitegraph_search` to find relevant endpoints, then `sitegraph_endpoint_detail`.
8. Use `sitegraph_history_search` only for authorized evidence and redact returned snippets.
9. Use `sitegraph_neighbors`, `sitegraph_trace`, or `sitegraph_shortest_path` with explicit depth/limits.
10. Export only necessary metadata or explicitly required exact evidence using `sitegraph_export`.
11. Delete or archive the SQLite file according to data retention policies after the engagement.

---

## 6. Site Map Terminology

- **Burp Target > Site map**: Burp's internal hierarchical tree of domains, directories, files, and parameterized requests.
- **Target `/sitemap.xml`**: An external web server resource optionally retrieved during web crawling.
- **Burp MCP sitegraph**: A local SQLite graph derived from bounded Burp metadata; enabling it does not crawl a target or make requests.
