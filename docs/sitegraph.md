# Sitegraph reference

It is a local SQLite metadata graph owned by Rust. It is separate from Burp's built-in **Target > Site map** and separate from a target's `/sitemap.xml` resource.

## Enable explicitly

Sitegraph is disabled by default. Enable it only when the target, retention policy, and local graph location are understood:

```toml
[sitegraph]
enabled = true
mode = "off"
project_root = "/absolute/path/to/burp-mcp/sitegraph"
rules_path = "/absolute/path/to/default-rules.json"
```

The file is loaded from `~/.config/burp-mcp/config.toml`; use `burp-mcp --config PATH serve` to select another file. The equivalent environment and CLI opt-in remain available:

```sh
BURP_MCP_ENABLE_SITEGRAPH=true burp-mcp serve
```


`project_root` is a directory, not a database filename. Burp MCP reads the active Burp project's stable `graph_id` and resolves the database as `<project_root>/<graph_id>.sqlite`; unsaved temporary projects use `<project_root>/temp-<graph_id>.sqlite`. Each project therefore has an independent database and daemon endpoint.
CLI flags and environment variables override TOML values. A project root or indexing mode does not enable sitegraph by itself. Restart the server after changing the enable flag.

On first SiteGraph enablement, Burp MCP initializes `~/.config/burp-mcp/default-rules.json` from its embedded rule pack. Edit that JSON to customize enrichment rules, or select another file with `[sitegraph].rules_path`. Existing rule files are validated and never overwritten. `sitegraph_config` only reports the effective runtime settings; it does not mutate them.

## Endpoint TLS

Set `[burp].tls = true` (or configure `tls_dir`) to make the resolved Burp endpoint use `https://`. If an `http://` endpoint is present in the file, the Rust client changes only its scheme and preserves host and port. The client then requires `ca.crt`, `client.crt`, and `client.key` in `tls_dir` or `~/.config/burp-mcp/tls`.

```toml
[burp]
endpoint = "http://burp-vm.test:9877"
tls = true
tls_dir = "/absolute/path/to/burp-mcp/tls"
```

The effective endpoint is `https://burp-vm.test:9877`.

## Shared daemon mode
The Rust-managed `sitegraph-daemon` owns one SQLite connection pool per project graph. MCP
processes connect to it through a loopback TCP endpoint described by a `0600`
endpoint file. The first `burp-mcp serve` instance for that project starts
the bundled daemon automatically; later instances for the same project reuse it.
Normally leave `[sitegraph].daemon` unset so the correct project daemon is selected automatically.
set `BURP_MCP_SITEGRAPH_DAEMON` or pass `--sitegraph-daemon PATH`.

The daemon is project-scoped, authenticated with a random per-startup token, and
uses newline-delimited JSON with bounded 128 MiB frames. Do not expose its
endpoint outside the local host or share one graph between unrelated projects.

## Data and privacy boundary

The graph stores normalized endpoint metadata, parameter names, relationships, observations, evidence summaries, and bounded provenance. It does not intentionally store parameter values or raw request/response bodies. Treat the SQLite database as security-sensitive metadata: apply local filesystem permissions, define retention, and remove it when the engagement ends.

Project identity determines graph partitioning. Do not point multiple unrelated Burp projects at a shared fallback database. Use a project-specific directory or explicit SQLite file and back it up only under the same authorization and data-handling policy as the Burp project.

## Operating modes

- `off`: no automatic sync. Manual `sitegraph_sync` remains available when enabled.
- `startup`: perform one bounded sync after startup.
- `watch`: periodically perform bounded syncs until shutdown. Sync failures are retained as operational errors and do not silently enable other tools.

Use `off` for reproducible investigations and invoke sync deliberately. Use `watch` only when continuous metadata collection is part of the engagement plan.

## Tool groups

The runtime schema is authoritative; inspect the exposed MCP tool definitions rather than copying fields from this guide.

- `sitegraph_sync`: import bounded Burp sitemap metadata into the local graph.
- `sitegraph_search`, `sitegraph_endpoint_detail`: locate normalized endpoints.
- `sitegraph_projects`, `sitegraph_stats`, `sitegraph_status`: inspect graph and sync state.
- `sitegraph_neighbors`, `sitegraph_trace`, `sitegraph_shortest_path`: traverse relationships with bounded limits.
- `sitegraph_clusters`, `sitegraph_impact`: group endpoints and inspect bounded impact.
- `sitegraph_diff`: compare graph observations between sync points.
- `sitegraph_export`: export metadata as JSON or CSV; review output before sharing.

## Recommended workflow

1. Confirm authorization and the exact Burp project/target.
2. Set a narrow Burp scope before collecting traffic.
3. Start with `--sitegraph-mode off`.
4. Review Burp HTTP history and Target > Site map first.
5. Run one bounded `sitegraph_sync` with a narrow URL prefix.
6. Inspect status and stats; verify the graph does not contain raw bodies or parameter values.
7. Use search and traversal tools with explicit pagination/depth limits.
8. Export only the metadata required for the report.
9. Delete or archive the SQLite graph according to the engagement retention policy.

## Site map terminology

- **Burp Target > Site map** is Burp's internal hierarchical view of domains, directories, files, and parameterized requests. It can be populated by proxy browsing, scanning, content discovery, and inferred content.
- **Target `/sitemap.xml`** is a server resource. A scanner may optionally request it during crawl discovery; it is not the same thing as Burp's internal map.
- **Burp MCP sitegraph** is a separate Rust-owned SQLite graph derived from bounded Burp metadata. Enabling it does not automatically crawl a target or request `/sitemap.xml`.

## References

- [Burp Target scope](https://portswigger.net/burp/documentation/desktop/tools/target/scope)
- [Burp Target site map](https://portswigger.net/burp/documentation/desktop/tools/target/site-map)
- [Burp HTTP history](https://portswigger.net/burp/documentation/desktop/tools/proxy/http-history)
- [Burp scanner crawl settings](https://portswigger.net/burp/documentation/scanner/scan-configurations/crawl-settings)
- [Burp MCP feature inventory](features.md)
- [Burp MCP implementation](../crates/sitegraph/)
