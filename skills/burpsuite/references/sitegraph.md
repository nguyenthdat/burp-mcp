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
| `--sitegraph-rules-path <PATH>` | `BURP_MCP_SITEGRAPH_RULES` | `~/.config/burp-mcp/default-rules.rules` | Sitegraph enrichment rules file (`.rules` DSL format). |

*Note: Merely specifying `--sitegraph-project-root` or `--sitegraph-mode` does not enable sitegraph without `--enable-sitegraph`.*

### 1.1 Enrichment Rule DSL

Enrichment rules are defined in declarative `.rules` files parsed by a custom Pest grammar. Once parsed into typed pack and rule structs, pattern evaluation executes via `regex::bytes::RegexSet` and `regex::bytes::Regex`, preserving byte-exact offsets and capture groups across arbitrary payloads without lossy UTF-8 conversion.

The DSL supports `pack` metadata blocks, `rule` blocks, raw string literals (`r"..."` or `r#"..."#`), standard escaped strings (`"..."`), capture group indexes, severity ratings (`critical`, `high`, `medium`, `low`), and surface filters (`request_message`, `response_message`, `response_body`, `websocket_payload`, `websocket_edited_payload`). Comments start with `#` or `//`.

Example `default-rules.rules` DSL snippet:

```text
pack {
    id = "burp-mcp-sitegraph"
    version = "2026.08.25"
    max_matches = 256
}

rule "jwt" {
    pattern = r"eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}"
    capture_group = 0
    severity = "high"
    surfaces = [
        "request_message",
        "response_message",
        "response_body",
        "websocket_payload",
        "websocket_edited_payload",
    ]
}
```

Legacy JSON rule files (`default-rules.json`) are strictly rejected following a clean cutover.

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
