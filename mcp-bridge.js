#!/usr/bin/env node
var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __hasOwnProp = Object.prototype.hasOwnProperty;
function __accessProp(key) {
  return this[key];
}
var __toCommonJS = (from) => {
  var entry = (__moduleCache ??= new WeakMap).get(from), desc;
  if (entry)
    return entry;
  entry = __defProp({}, "__esModule", { value: true });
  if (from && typeof from === "object" || typeof from === "function") {
    for (var key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(entry, key))
        __defProp(entry, key, {
          get: __accessProp.bind(from, key),
          enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable
        });
  }
  __moduleCache.set(from, entry);
  return entry;
};
var __moduleCache;
var __returnValue = (v) => v;
function __exportSetter(name, newValue) {
  this[name] = __returnValue.bind(null, newValue);
}
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, {
      get: all[name],
      enumerable: true,
      configurable: true,
      set: __exportSetter.bind(all, name)
    });
};

// bridge/src/main.ts
var exports_main = {};
__export(exports_main, {
  main: () => main
});
module.exports = __toCommonJS(exports_main);

// bridge/src/burp-provider.ts
var import_node_http = require("node:http");

// bridge/src/burp-tool-descriptions.ts
var DESCRIPTIONS = {
  proxy_history: "Get Burp proxy history with optional filtering by URL, method, status code",
  proxy_detail: "Get full request/response details for a specific proxy history item by index",
  proxy_websocket: "Get WebSocket message history",
  proxy_clear: "Clear proxy history",
  proxy_history_filtered: "Filter proxy history by annotation color or notes",
  send_request: "Send an HTTP request through Burp and get the response",
  send_to_repeater: "Display a raw HTTP request in Burp Repeater without sending it. tab_name is an optional tab caption, not a tag.",
  repeater_send: "Send a request and get response (like Repeater)",
  repeater_modify_send: "Modify headers/body of a request then send it",
  send_to_intruder: "Send a request to Burp Intruder",
  intruder_attack: "Run a numeric range brute force attack (synchronous)",
  intruder_attack_async: "Run a multi-threaded numeric range brute force attack",
  intruder_attack_wordlist: "Run a wordlist-based attack",
  intruder_pitchfork: "Run a Pitchfork attack (parallel multi-param)",
  intruder_cluster_bomb: "Run a Cluster Bomb attack (cartesian product multi-param)",
  intruder_battering_ram: "Run a Battering Ram attack (same payload all positions)",
  intruder_with_options: "Run attack with advanced options (throttle, encoding, grep, timing)",
  sitemap: "Get site map entries with optional URL prefix filter",
  target_info: "Get target information (hosts, technologies detected)",
  intercept_toggle: "Enable or disable proxy intercept",
  encode: "Encode a string (base64, url, hex)",
  decode: "Decode a string (base64, url)",
  convert_request: "Convert HTTP request method (e.g. GET to POST)",
  export_request: "Export a request as curl command",
  generate_csrf_poc: "Generate a CSRF proof-of-concept HTML page",
  extract_from_response: "Extract data from a response using regex",
  payload_process: "Process a payload (hash, encode, reverse, etc.)",
  scan: "Start a vulnerability scan",
  scan_active: "Start a standard legacy active audit for a request. This cannot target or prove execution of a particular BCheck; poll scan_results for discovered issues. Requires Burp Professional.",
  scan_results: "Get scan results (discovered vulnerabilities)",
  scan_issue_detail: "Get detailed information about a specific scan issue",
  crawl: "Start crawling a URL (adds to scope)",
  get_scope: "Check if a URL is in Burp scope",
  add_to_scope: "Add a URL to Burp scope",
  remove_from_scope: "Remove a URL from Burp scope",
  collaborator_generate: "Generate Burp Collaborator payloads for OOB testing",
  collaborator_poll: "Poll for Collaborator interactions (DNS/HTTP callbacks)",
  search_history: "Search proxy history with regex (in URL, request, or response)",
  highlight: "Set the highlight color on an item in the current Burp Proxy HTTP history. This does not address Repeater history or tags.",
  annotate: "Set notes on an item in the current Burp Proxy HTTP history. This does not address Repeater history or tags.",
  compare: "Compare two proxy history responses (diff)",
  export_config: "Export Burp project configuration as JSON",
  import_config: "Import Burp project configuration from JSON",
  set_upstream_proxy: "Set upstream proxy (SOCKS/HTTP) for all Burp traffic",
  set_dns_override: "Override DNS resolution for a hostname",
  set_http2: "Enable or disable HTTP/2",
  cookie_jar: "View cookies in Burp cookie jar (with optional domain filter)",
  token_analysis: "Analyze token entropy and randomness",
  sequencer: "Analyze a batch of tokens for randomness quality",
  save_project: "Save the current Burp project",
  burp_version: "Get Burp Suite version information",
  add_issue: "Manually add a vulnerability issue to the site map",
  register_http_handler: "Register an auto-modify rule for HTTP requests (add header or replace text)",
  remove_http_handler: "Remove/clear HTTP handler rules",
  register_proxy_rule: "Register a proxy intercept rule (intercept URLs containing a string)",
  remove_proxy_rule: "Remove/clear proxy intercept rules",
  extensions_list: "Get information about loaded extensions",
  log: "Write a message to Burp extension output log",
  audit_log: "View audit log entries",
  privacy_mode: "Set privacy mode (strict/off)",
  scope_gate: "Enable/disable scope gate",
  inline_fuzzer: "FUZZ marker fuzzing",
  race_condition: "Race condition testing",
  access_control_sweep: "Test different auth levels",
  injection_probe: "SQLi/SSTI/LFI probe",
  jwt_attack: "JWT attacks (alg:none)",
  jwt_decode: "Decode JWT tokens",
  session_remove_rule: "Remove session rule",
  session_list_rules: "List session rules",
  session_create_rule: "Create session handling rule",
  passive_intel: "Extract secrets from proxy history",
  websocket_list: "List active WebSockets",
  websocket_close: "Close WebSocket",
  websocket_send_binary: "Send binary on WebSocket",
  websocket_send_text: "Send text on WebSocket",
  websocket_create: "Create WebSocket connection",
  send_request_parallel: "Send parallel HTTP requests",
  cookie_jar_set: "Set cookie in Burp cookie jar",
  bambda_import: "Import a Bambda script into Burp. Imports only; this tool does not execute the script.",
  bcheck_import: "Import a BCheck script into Burp with the requested enabled state. Imports only; this tool does not run the BCheck."
};
function getBurpToolDescription(name) {
  return DESCRIPTIONS[name] ?? `Burp Suite tool: ${name}`;
}

// bridge/src/burp-tool-schemas.ts
var REQUIRED = {
  bambda_import: ["script"],
  bcheck_import: ["script", "enabled"],
  send_to_repeater: ["request"],
  scan_active: ["host", "request"],
  highlight: ["index"],
  annotate: ["index", "note"]
};
var SCHEMAS = {
  proxy_history: {
    limit: { type: "number", description: "Max items (default 100)" },
    offset: { type: "number" },
    url_filter: { type: "string" },
    method_filter: { type: "string" },
    status_filter: { type: "number" }
  },
  proxy_detail: { index: { type: "number", description: "History item index" } },
  proxy_websocket: { limit: { type: "number" } },
  proxy_history_filtered: {
    has_notes: { type: "string" },
    color: { type: "string" },
    limit: { type: "number" }
  },
  send_request: {
    method: { type: "string" },
    url: { type: "string", description: "Full URL" },
    body: { type: "string" },
    headers: { type: "object" }
  },
  repeater_send: {
    request: { type: "string", description: "Raw HTTP request" },
    host: { type: "string" },
    port: { type: "number" },
    https: { type: "boolean" }
  },
  repeater_modify_send: {
    request: { type: "string" },
    host: { type: "string" },
    port: { type: "number" },
    https: { type: "boolean" },
    replace_header: { type: "object" },
    add_header: { type: "object" },
    replace_body: { type: "string" }
  },
  send_to_repeater: {
    request: {
      type: "string",
      description: "Raw HTTP request displayed in Repeater without sending it"
    },
    tab_name: {
      type: "string",
      description: "Optional Repeater tab caption, not a tag",
      default: "MCP"
    }
  },
  send_to_intruder: { request: { type: "string" } },
  intruder_attack: {
    url_template: { type: "string", description: "URL with @@ placeholder" },
    from: { type: "number" },
    to: { type: "number" },
    pad_digits: { type: "number" },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" },
    success_contains: { type: "string" }
  },
  intruder_attack_async: {
    url_template: { type: "string" },
    from: { type: "number" },
    to: { type: "number" },
    pad_digits: { type: "number" },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" },
    threads: { type: "number" }
  },
  intruder_attack_wordlist: {
    url_template: { type: "string" },
    wordlist: { type: "array", items: { type: "string" } },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" },
    body_template: { type: "string" }
  },
  intruder_pitchfork: {
    url_template: { type: "string" },
    placeholders: { type: "object" },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" }
  },
  intruder_cluster_bomb: {
    url_template: { type: "string" },
    placeholders: { type: "object" },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" },
    max_requests: { type: "number" }
  },
  intruder_battering_ram: {
    url_template: { type: "string" },
    wordlist: { type: "array", items: { type: "string" } },
    placeholder: { type: "string" },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" }
  },
  intruder_with_options: {
    url_template: { type: "string" },
    from: { type: "number" },
    to: { type: "number" },
    pad_digits: { type: "number" },
    method: { type: "string" },
    headers: { type: "object" },
    success_length_not: { type: "number" },
    throttle_ms: { type: "number" },
    payload_prefix: { type: "string" },
    payload_suffix: { type: "string" },
    payload_encoding: { type: "string" },
    grep_extract: { type: "string" },
    record_time: { type: "boolean" }
  },
  sitemap: { url_prefix: { type: "string" }, limit: { type: "number" } },
  target_info: { url: { type: "string" } },
  intercept_toggle: { enable: { type: "boolean" } },
  encode: {
    input: { type: "string" },
    type: { type: "string", description: "base64, url, or hex" }
  },
  decode: { input: { type: "string" }, type: { type: "string", description: "base64 or url" } },
  convert_request: { request: { type: "string" }, convert_to: { type: "string" } },
  export_request: {
    request: { type: "string" },
    host: { type: "string" },
    format: { type: "string", description: "curl or python" },
    https: { type: "boolean" }
  },
  generate_csrf_poc: {
    request: { type: "string" },
    host: { type: "string" },
    https: { type: "boolean" }
  },
  extract_from_response: { index: { type: "number" }, regex: { type: "string" } },
  payload_process: {
    input: { type: "string" },
    operation: {
      type: "string",
      description: "base64_encode/decode, url_encode/decode, md5, sha1, sha256, hex_encode, lowercase, uppercase, reverse, length"
    }
  },
  scan_active: {
    request: {
      type: "string",
      description: "Raw HTTP request that seeds a standard active audit and does not select a BCheck"
    },
    host: { type: "string" },
    port: { type: "number" },
    https: { type: "boolean" }
  },
  scan_results: { limit: { type: "number" } },
  scan_issue_detail: { index: { type: "number" } },
  crawl: { url: { type: "string" } },
  get_scope: { url: { type: "string" } },
  add_to_scope: { url: { type: "string" } },
  remove_from_scope: { url: { type: "string" } },
  collaborator_generate: { count: { type: "number" } },
  search_history: {
    regex: { type: "string" },
    search_in: { type: "string", description: "url, request, or response" },
    limit: { type: "number" }
  },
  highlight: {
    index: { type: "number", description: "Zero-based current Proxy HTTP history index" },
    color: { type: "string" }
  },
  annotate: {
    index: { type: "number", description: "Zero-based current Proxy HTTP history index" },
    note: { type: "string" }
  },
  compare: { index1: { type: "number" }, index2: { type: "number" } },
  import_config: { config: { type: "string" } },
  set_upstream_proxy: {
    proxy_host: { type: "string" },
    proxy_port: { type: "number" },
    type: { type: "string" }
  },
  set_dns_override: { hostname: { type: "string" }, ip: { type: "string" } },
  set_http2: { enable: { type: "boolean" } },
  cookie_jar: { limit: { type: "number" }, domain: { type: "string" } },
  token_analysis: { tokens: { type: "array", items: { type: "string" } } },
  sequencer: { tokens: { type: "array", items: { type: "string" } } },
  add_issue: {
    name: { type: "string" },
    url: { type: "string" },
    detail: { type: "string" },
    severity: { type: "string" },
    confidence: { type: "string" }
  },
  register_http_handler: {
    header_name: { type: "string" },
    header_value: { type: "string" },
    match: { type: "string" },
    replace: { type: "string" }
  },
  register_proxy_rule: { url_contains: { type: "string" } },
  log: { message: { type: "string" }, level: { type: "string" } },
  audit_log: { limit: { type: "number" } },
  privacy_mode: { mode: { type: "string" } },
  scope_gate: { action: { type: "string" } },
  inline_fuzzer: {
    template: { type: "string" },
    host: { type: "string" },
    wordlist: { type: "array" }
  },
  race_condition: {
    request: { type: "string" },
    host: { type: "string" },
    count: { type: "number" }
  },
  access_control_sweep: {
    request: { type: "string" },
    host: { type: "string" },
    auth_headers: { type: "string" }
  },
  injection_probe: { url: { type: "string" }, param: { type: "string" }, type: { type: "string" } },
  jwt_attack: { token: { type: "string" }, attack: { type: "string" } },
  jwt_decode: { token: { type: "string" } },
  session_remove_rule: {},
  session_list_rules: {},
  session_create_rule: { find: { type: "string" }, replace: { type: "string" } },
  passive_intel: { limit: { type: "number" } },
  websocket_list: {},
  websocket_close: { id: { type: "string" } },
  websocket_send_binary: { id: { type: "string" }, data: { type: "string" } },
  websocket_send_text: { id: { type: "string" }, text: { type: "string" } },
  websocket_create: { host: { type: "string" }, port: { type: "number" } },
  send_request_parallel: { requests: { type: "array" } },
  cookie_jar_set: { url: { type: "string" }, name: { type: "string" }, value: { type: "string" } },
  bambda_import: {
    script: {
      type: "string",
      description: "Complete Bambda script to import only; it is not executed"
    }
  },
  bcheck_import: {
    script: { type: "string", description: "Complete BCheck script to import only; it is not run" },
    enabled: { type: "boolean", description: "Requested BCheck state after a successful import" }
  }
};
function getBurpToolInputSchema(name) {
  const properties = SCHEMAS[name] ?? {};
  const required = REQUIRED[name];
  return required === undefined ? { type: "object", properties } : { type: "object", properties, required };
}

// bridge/src/json.ts
class InvalidJsonError extends Error {
  name = "InvalidJsonError";
}
function parseJson(text) {
  let value;
  try {
    value = JSON.parse(text);
  } catch (error) {
    throw new InvalidJsonError("Invalid JSON", { cause: error });
  }
  if (!isJsonValue(value)) {
    throw new InvalidJsonError("JSON contains an unsupported value");
  }
  return value;
}
function isJsonValue(value) {
  const pending = [value];
  while (pending.length > 0) {
    const current = pending.pop();
    if (current === null || typeof current === "string" || typeof current === "boolean") {
      continue;
    }
    if (typeof current === "number") {
      if (!Number.isFinite(current)) {
        return false;
      }
      continue;
    }
    if (Array.isArray(current)) {
      for (const item of current) {
        pending.push(item);
      }
      continue;
    }
    if (typeof current !== "object") {
      return false;
    }
    for (const item of Object.values(current)) {
      pending.push(item);
    }
  }
  return true;
}

// bridge/src/types.ts
class ProviderUnavailableError extends Error {
  namespace;
  publicMessage;
  name = "ProviderUnavailableError";
  constructor(namespace, publicMessage, options) {
    super(publicMessage, options);
    this.namespace = namespace;
    this.publicMessage = publicMessage;
  }
}
function isJsonObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
function hasOwnError(value) {
  return isJsonObject(value) && Object.hasOwn(value, "error");
}

// bridge/src/burp-provider.ts
class BurpConnectionError extends Error {
  name = "BurpConnectionError";
}

class BurpDiscoveryError extends ProviderUnavailableError {
  name = "BurpDiscoveryError";
  constructor(config, options) {
    super("burp", `Burp MCP not connected at ${config.host}:${config.port}. Start Burp with the "Burp MCP" extension loaded, then retry.`, options);
  }
}

class BurpHttpProvider {
  config;
  diagnostics;
  namespace = "burp";
  tools;
  toolsRequest;
  constructor(config, diagnostics = process.stderr) {
    this.config = config;
    this.diagnostics = diagnostics;
  }
  async listTools() {
    if (this.tools !== undefined) {
      return this.tools;
    }
    if (this.toolsRequest === undefined) {
      this.toolsRequest = this.fetchToolNames().then((names) => {
        const tools = names.map((localName) => ({
          localName,
          description: getBurpToolDescription(localName),
          inputSchema: getBurpToolInputSchema(localName)
        }));
        this.tools = tools;
        this.diagnostics.write(`[burp-mcp-bridge] Connected to Burp. ${tools.length} tools available.
`);
        return tools;
      }).finally(() => {
        this.toolsRequest = undefined;
      });
    }
    return this.toolsRequest;
  }
  async callTool(localName, arguments_) {
    const body = JSON.stringify({ tool: localName, params: arguments_ });
    return new Promise((resolve, reject) => {
      const outgoing = import_node_http.request({
        hostname: this.config.host,
        port: this.config.port,
        path: "/",
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(body),
          ...this.config.authHeaders
        }
      }, (response) => {
        let data = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          data += chunk;
        });
        response.on("end", () => {
          const statusCode = response.statusCode ?? 0;
          if (statusCode === 403) {
            resolve({
              error: "Burp MCP rejected the request: missing or invalid BURP_MCP_TOKEN. Set BURP_MCP_TOKEN to match the token in ~/.burp-mcp-token."
            });
            return;
          }
          const parsed = parseBackendResponse(data);
          if (statusCode < 200 || statusCode >= 300) {
            if (parsed.isJson && hasOwnError(parsed.value)) {
              resolve(parsed.value);
              return;
            }
            resolve({ error: `Burp MCP returned HTTP ${statusCode}: ${data.slice(0, 200)}` });
            return;
          }
          resolve(parsed.value);
        });
      });
      outgoing.on("error", (error) => {
        reject(new BurpConnectionError(`Cannot reach Burp MCP at ${this.config.host}:${this.config.port} (${getErrorCode(error) ?? error.message}). ` + 'Ensure Burp Suite is running with the "Burp MCP" extension loaded. ' + "If the port differs, set BURP_MCP_PORT and -Dburp.mcp.port=<same> on Burp.", { cause: error }));
      });
      outgoing.write(body);
      outgoing.end();
    });
  }
  invalidate() {
    this.tools = undefined;
  }
  async fetchToolNames() {
    const value = await new Promise((resolve, reject) => {
      const outgoing = import_node_http.request({
        hostname: this.config.host,
        port: this.config.port,
        path: "/tools",
        method: "GET",
        headers: this.config.authHeaders
      }, (response) => {
        let data = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          data += chunk;
        });
        response.on("end", () => {
          if (response.statusCode !== 200) {
            reject(new BurpDiscoveryError(this.config, {
              cause: new Error(`Cannot fetch tools: HTTP ${response.statusCode ?? 0} ${data.slice(0, 200)}`)
            }));
            return;
          }
          try {
            resolve(parseJson(data));
          } catch (error) {
            reject(new BurpDiscoveryError(this.config, { cause: error }));
          }
        });
      });
      outgoing.on("error", (error) => {
        reject(new BurpDiscoveryError(this.config, { cause: error }));
      });
      outgoing.end();
    });
    if (!Array.isArray(value) || !value.every((name) => typeof name === "string")) {
      throw new BurpDiscoveryError(this.config, {
        cause: new InvalidJsonError("Burp /tools must return an array of strings")
      });
    }
    return value;
  }
}
function parseBackendResponse(data) {
  try {
    return { isJson: true, value: parseJson(data) };
  } catch (error) {
    if (error instanceof InvalidJsonError) {
      return { isJson: false, value: { error: data } };
    }
    throw error;
  }
}
function getErrorCode(error) {
  const code = Reflect.get(error, "code");
  return typeof code === "string" ? code : undefined;
}

// bridge/src/config.ts
var import_node_fs = require("node:fs");
var import_node_os = require("node:os");
var import_node_path = require("node:path");
function loadConfig(env = process.env) {
  const host = env["BURP_MCP_HOST"] || "127.0.0.1";
  const port = Number.parseInt(env["BURP_MCP_PORT"] || "9876", 10);
  const token = resolveToken(env);
  return {
    host,
    port,
    authHeaders: token === null || token.length === 0 ? {} : { Authorization: `Bearer ${token}` }
  };
}
function resolveToken(env) {
  const environmentToken = env["BURP_MCP_TOKEN"];
  if (environmentToken) {
    return environmentToken;
  }
  try {
    const tokenFile = import_node_path.join(import_node_os.homedir(), ".burp-mcp-token");
    return import_node_fs.existsSync(tokenFile) ? import_node_fs.readFileSync(tokenFile, "utf8").trim() : null;
  } catch (error) {
    if (error instanceof Error) {
      return null;
    }
    throw error;
  }
}

// bridge/src/rpc.ts
class RpcDispatcher {
  directory;
  constructor(directory) {
    this.directory = directory;
  }
  async handle(message) {
    const request2 = isJsonObject(message) ? message : {};
    const method = request2["method"];
    const id = request2["id"] ?? null;
    switch (method) {
      case "initialize":
        return {
          jsonrpc: "2.0",
          id,
          result: {
            protocolVersion: "2024-11-05",
            capabilities: { tools: {} },
            serverInfo: { name: "burpsuite-mcp", version: "2.0.0" }
          }
        };
      case "notifications/initialized":
        return null;
      case "tools/list":
        return this.listTools(id);
      case "tools/call":
        return this.callTool(id, request2["params"]);
      default:
        return rpcError(id, -32601, `Method not found: ${String(method)}`);
    }
  }
  async listTools(id) {
    try {
      return { jsonrpc: "2.0", id, result: { tools: await this.directory.listTools() } };
    } catch (error) {
      if (error instanceof ProviderUnavailableError) {
        return rpcError(id, -32000, error.publicMessage);
      }
      throw error;
    }
  }
  async callTool(id, rawParams) {
    if (!isJsonObject(rawParams) || typeof rawParams["name"] !== "string") {
      return rpcError(id, -32602, "tools/call requires params.name");
    }
    const name = rawParams["name"];
    const rawArguments = rawParams["arguments"];
    const arguments_ = rawArguments ? rawArguments : {};
    try {
      const result = await this.directory.callTool(name, arguments_);
      const toolResult = {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        ...hasOwnError(result) ? { isError: true } : {}
      };
      return { jsonrpc: "2.0", id, result: toolResult };
    } catch (error) {
      if (error instanceof ProviderUnavailableError) {
        return rpcError(id, -32000, error.publicMessage);
      }
      this.directory.invalidate(name);
      return rpcError(id, -1, error instanceof Error ? error.message : "Tool call failed");
    }
  }
}
function rpcError(id, code, message) {
  return { jsonrpc: "2.0", id, error: { code, message } };
}

// bridge/src/stdio.ts
var import_node_readline = require("node:readline");
function runStdio(dispatcher, input = process.stdin, output = process.stdout) {
  const lines = import_node_readline.createInterface({ input, terminal: false });
  const writer = new LineWriter(output);
  let pending = 0;
  let inputClosed = false;
  let resolveCompletion;
  const completion = new Promise((resolve) => {
    resolveCompletion = resolve;
  });
  const finishWhenDrained = () => {
    if (!inputClosed || pending !== 0) {
      return;
    }
    writer.drain().then(() => resolveCompletion?.());
  };
  lines.on("line", (line) => {
    if (!line.trim()) {
      return;
    }
    let message;
    try {
      message = parseJson(line);
    } catch (error) {
      if (error instanceof InvalidJsonError) {
        writer.write(`${JSON.stringify(rpcError(null, -32700, "Parse error"))}
`);
        return;
      }
      throw error;
    }
    pending += 1;
    dispatcher.handle(message).then((response) => {
      if (response !== null) {
        writer.write(`${JSON.stringify(response)}
`);
      }
    }).catch((error) => {
      const id = isJsonObject(message) ? message["id"] ?? null : null;
      const errorMessage = error instanceof Error ? error.message : "Handler error";
      writer.write(`${JSON.stringify(rpcError(id, -1, errorMessage))}
`);
    }).finally(() => {
      pending -= 1;
      finishWhenDrained();
    });
  });
  lines.on("close", () => {
    inputClosed = true;
    finishWhenDrained();
  });
  return completion;
}

class LineWriter {
  output;
  queue = Promise.resolve();
  constructor(output) {
    this.output = output;
  }
  write(line) {
    this.queue = this.queue.then(() => new Promise((resolve) => {
      if (this.output.write(line)) {
        resolve();
      } else {
        this.output.once("drain", resolve);
      }
    }));
  }
  async drain() {
    await this.queue;
  }
}

// bridge/src/tool-directory.ts
class ToolDirectoryError extends Error {
  name = "ToolDirectoryError";
}

class ToolDirectory {
  providers;
  compatibilityNamespace;
  providersByNamespace;
  providersByPrefixLength;
  constructor(providers, compatibilityNamespace) {
    this.providers = providers;
    this.compatibilityNamespace = compatibilityNamespace;
    const providersByNamespace = new Map;
    for (const provider of providers) {
      if (providersByNamespace.has(provider.namespace)) {
        throw new ToolDirectoryError(`Duplicate tool provider namespace: ${provider.namespace}`);
      }
      providersByNamespace.set(provider.namespace, provider);
    }
    if (!providersByNamespace.has(compatibilityNamespace)) {
      throw new ToolDirectoryError(`Missing compatibility provider namespace: ${compatibilityNamespace}`);
    }
    this.providersByNamespace = providersByNamespace;
    this.providersByPrefixLength = Array.from(providers).sort((left, right) => right.namespace.length - left.namespace.length);
  }
  async listTools() {
    const settledProviderTools = await Promise.allSettled(this.providers.map(async (provider) => ({ provider, tools: await provider.listTools() })));
    const providerTools = [];
    const unavailableProviders = [];
    for (const result of settledProviderTools) {
      if (result.status === "fulfilled") {
        providerTools.push(result.value);
      } else if (result.reason instanceof ProviderUnavailableError) {
        unavailableProviders.push(result.reason);
      } else {
        throw result.reason;
      }
    }
    if (providerTools.length === 0) {
      const firstUnavailable = unavailableProviders[0];
      if (firstUnavailable !== undefined) {
        throw firstUnavailable;
      }
    }
    const names = new Set;
    return providerTools.flatMap(({ provider, tools }) => tools.map((tool) => {
      const name = `${provider.namespace}_${tool.localName}`;
      if (names.has(name)) {
        throw new ToolDirectoryError(`Duplicate public tool name: ${name}`);
      }
      names.add(name);
      return { name, description: tool.description, inputSchema: tool.inputSchema };
    }));
  }
  async callTool(publicName, arguments_) {
    const resolved = this.resolve(publicName);
    await resolved.provider.listTools();
    return resolved.provider.callTool(resolved.localName, arguments_);
  }
  invalidate(publicName) {
    this.resolve(publicName).provider.invalidate();
  }
  resolve(publicName) {
    for (const provider2 of this.providersByPrefixLength) {
      const prefix = `${provider2.namespace}_`;
      if (publicName.startsWith(prefix)) {
        return { provider: provider2, localName: publicName.slice(prefix.length) };
      }
    }
    const provider = this.providersByNamespace.get(this.compatibilityNamespace);
    if (provider === undefined) {
      throw new ToolDirectoryError(`Missing compatibility provider namespace: ${this.compatibilityNamespace}`);
    }
    return { provider, localName: publicName };
  }
}

// bridge/src/main.ts
async function main() {
  const config = loadConfig();
  const burp = new BurpHttpProvider(config);
  const directory = new ToolDirectory([burp], "burp");
  try {
    await burp.listTools();
  } catch (error) {
    if (error instanceof BurpDiscoveryError) {
      process.stderr.write(`[burp-mcp-bridge] WARNING: Cannot connect to Burp at ${config.host}:${config.port}. Start Burp first.
`);
    } else {
      throw error;
    }
  }
  await runStdio(new RpcDispatcher(directory));
}
main();
