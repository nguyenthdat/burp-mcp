'use strict';

const assert = require('node:assert/strict');
const { once } = require('node:events');
const http = require('node:http');
const net = require('node:net');
const { spawn } = require('node:child_process');
const { test } = require('node:test');
const path = require('node:path');

class LineReader {
  constructor(stream) {
    this.lines = [];
    this.waiters = [];
    this.buffer = '';
    stream.setEncoding('utf8');
    stream.on('data', chunk => {
      this.buffer += chunk;
      const parts = this.buffer.split('\n');
      this.buffer = parts.pop();
      for (const line of parts) {
        this.push(line.replace(/\r$/, ''));
      }
    });
  }

  push(line) {
    const waiter = this.waiters.shift();
    if (waiter) {
      waiter.resolve(line);
    } else {
      this.lines.push(line);
    }
  }

  next(timeoutMs = 5000) {
    if (this.lines.length > 0) return Promise.resolve(this.lines.shift());
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        const index = this.waiters.indexOf(waiter);
        if (index >= 0) this.waiters.splice(index, 1);
        reject(new Error('Timed out waiting for a line'));
      }, timeoutMs);
      const waiter = {
        resolve: line => {
          clearTimeout(timeout);
          resolve(line);
        }
      };
      this.waiters.push(waiter);
    });
  }
}

function getFreePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

function close(server) {
  if (!server) return Promise.resolve();
  return new Promise(resolve => server.close(() => resolve()));
}

test('reconnects after Burp starts and handles a later tool call', async () => {
  const port = await getFreePort();
  const bridgePath = path.join(__dirname, '..', 'mcp-bridge.js');
  const bridge = spawn(process.execPath, [bridgePath], {
    env: {
      ...process.env,
      BURP_MCP_HOST: '127.0.0.1',
      BURP_MCP_PORT: String(port),
      BURP_MCP_TOKEN: 'test-token'
    },
    stdio: ['pipe', 'pipe', 'pipe']
  });
  const stdout = new LineReader(bridge.stdout);
  const stderr = new LineReader(bridge.stderr);
  let server;
  const requests = [];

  try {
    const warning = await stderr.next();
    assert.match(warning, /Cannot connect to Burp/);

    server = http.createServer((request, response) => {
      if (request.method === 'GET' && request.url === '/tools') {
        response.writeHead(200, { 'Content-Type': 'application/json' });
        response.end(JSON.stringify(['proxy_history']));
        return;
      }

      let body = '';
      request.on('data', chunk => { body += chunk; });
      request.on('end', () => {
        requests.push(JSON.parse(body));
        response.writeHead(200, { 'Content-Type': 'application/json' });
        response.end(JSON.stringify({ ok: true }));
      });
    });
    await new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(port, '127.0.0.1', resolve);
    });

    bridge.stdin.write(JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/list',
      params: {}
    }) + '\n');
    const listResponse = JSON.parse(await stdout.next());
    assert.equal(listResponse.id, 1);
    assert.equal(listResponse.result.tools[0].name, 'burp_proxy_history');

    bridge.stdin.write(JSON.stringify({
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: { name: 'burp_proxy_history', arguments: { limit: 1 } }
    }) + '\n');
    const callResponse = JSON.parse(await stdout.next());
    assert.equal(callResponse.id, 2);
    assert.equal('isError' in callResponse.result, false);
    assert.deepEqual(requests[0], { tool: 'proxy_history', params: { limit: 1 } });
    assert.match(callResponse.result.content[0].text, /"ok": true/);
  } finally {
    const childClosed = bridge.exitCode === null && bridge.signalCode === null
      ? once(bridge, 'close')
      : Promise.resolve();
    bridge.stdin.end();
    bridge.kill();
    await close(server);
    await childClosed.catch(() => {});
  }
});

test('advertises import contracts and forwards multiline scripts exactly', async () => {
  const port = await getFreePort();
  const bridgePath = path.join(__dirname, '..', 'mcp-bridge.js');
  const requests = [];
  const toolNames = [
    'bambda_import',
    'bcheck_import',
    'send_to_repeater',
    'scan_active',
    'highlight',
    'annotate',
  ];
  const server = http.createServer((request, response) => {
    if (request.method === 'GET' && request.url === '/tools') {
      response.writeHead(200, { 'Content-Type': 'application/json' });
      response.end(JSON.stringify(toolNames));
      return;
    }

    let body = '';
    request.on('data', chunk => { body += chunk; });
    request.on('end', () => {
      requests.push(JSON.parse(body));
      response.writeHead(200, { 'Content-Type': 'application/json' });
      response.end(JSON.stringify({ imported: requests.at(-1).tool }));
    });
  });
  const bridge = spawn(process.execPath, [bridgePath], {
    env: {
      ...process.env,
      BURP_MCP_HOST: '127.0.0.1',
      BURP_MCP_PORT: String(port),
      BURP_MCP_TOKEN: 'test-token'
    },
    stdio: ['pipe', 'pipe', 'pipe']
  });
  const stdout = new LineReader(bridge.stdout);

  try {
    await new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(port, '127.0.0.1', resolve);
    });

    bridge.stdin.write(JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/list',
      params: {}
    }) + '\n');
    const listResponse = JSON.parse(await stdout.next());
    const toolsByName = new Map(listResponse.result.tools.map(tool => [tool.name, tool]));

    assert.equal(
      toolsByName.get('burp_bambda_import').description,
      'Import a Bambda script into Burp. Imports only; this tool does not execute the script.'
    );
    assert.deepEqual(toolsByName.get('burp_bambda_import').inputSchema, {
      type: 'object',
      properties: {
        script: {
          type: 'string',
          description: 'Complete Bambda script to import only; it is not executed'
        }
      },
      required: ['script']
    });
    assert.equal(
      toolsByName.get('burp_bcheck_import').description,
      'Import a BCheck script into Burp with the requested enabled state. Imports only; this tool does not run the BCheck.'
    );
    assert.deepEqual(toolsByName.get('burp_bcheck_import').inputSchema, {
      type: 'object',
      properties: {
        script: {
          type: 'string',
          description: 'Complete BCheck script to import only; it is not run'
        },
        enabled: {
          type: 'boolean',
          description: 'Requested BCheck state after a successful import'
        }
      },
      required: ['script', 'enabled']
    });
    assert.deepEqual(toolsByName.get('burp_send_to_repeater').inputSchema.required, ['request']);
    assert.equal(
      toolsByName.get('burp_send_to_repeater').description,
      'Display a raw HTTP request in Burp Repeater without sending it. tab_name is an optional tab caption, not a tag.'
    );
    assert.deepEqual(toolsByName.get('burp_send_to_repeater').inputSchema.properties, {
      request: {
        type: 'string',
        description: 'Raw HTTP request displayed in Repeater without sending it'
      },
      tab_name: {
        type: 'string',
        description: 'Optional Repeater tab caption, not a tag',
        default: 'MCP'
      }
    });
    assert.equal(
      toolsByName.get('burp_scan_active').description,
      'Start a standard legacy active audit for a request. This cannot target or prove execution of a particular BCheck; poll scan_results for discovered issues. Requires Burp Professional.'
    );
    assert.deepEqual(toolsByName.get('burp_scan_active').inputSchema.properties, {
      request: {
        type: 'string',
        description: 'Raw HTTP request that seeds a standard active audit and does not select a BCheck'
      },
      host: { type: 'string' },
      port: { type: 'number' },
      https: { type: 'boolean' }
    });
    assert.deepEqual(toolsByName.get('burp_scan_active').inputSchema.required, ['host', 'request']);
    assert.equal(
      toolsByName.get('burp_highlight').description,
      'Set the highlight color on an item in the current Burp Proxy HTTP history. This does not address Repeater history or tags.'
    );
    assert.equal(
      toolsByName.get('burp_highlight').inputSchema.properties.index.description,
      'Zero-based current Proxy HTTP history index'
    );
    assert.deepEqual(toolsByName.get('burp_highlight').inputSchema.required, ['index']);
    assert.equal(
      toolsByName.get('burp_annotate').description,
      'Set notes on an item in the current Burp Proxy HTTP history. This does not address Repeater history or tags.'
    );
    assert.equal(
      toolsByName.get('burp_annotate').inputSchema.properties.index.description,
      'Zero-based current Proxy HTTP history index'
    );
    assert.deepEqual(toolsByName.get('burp_annotate').inputSchema.required, ['index', 'note']);

    const bambdaScript = 'if (request.method() == "GET") {\n  return "quoted value";\n}\n';
    bridge.stdin.write(JSON.stringify({
      jsonrpc: '2.0',
      id: 2,
      method: 'tools/call',
      params: { name: 'burp_bambda_import', arguments: { script: bambdaScript } }
    }) + '\n');
    const bambdaResponse = JSON.parse(await stdout.next());
    assert.equal('isError' in bambdaResponse.result, false);
    assert.deepEqual(requests[0], {
      tool: 'bambda_import',
      params: { script: bambdaScript }
    });
    assert.deepEqual(bambdaResponse.result.content, [{
      type: 'text',
      text: JSON.stringify({ imported: 'bambda_import' }, null, 2)
    }]);

    const bcheckScript = 'metadata:\n  language: v2-beta\n  name: "quoted BCheck"\n\ngiven request then\n  report issue:\n    detail: "line one\\nline two"\n';
    bridge.stdin.write(JSON.stringify({
      jsonrpc: '2.0',
      id: 3,
      method: 'tools/call',
      params: {
        name: 'burp_bcheck_import',
        arguments: { script: bcheckScript, enabled: false }
      }
    }) + '\n');
    const bcheckResponse = JSON.parse(await stdout.next());
    assert.equal('isError' in bcheckResponse.result, false);
    assert.deepEqual(requests[1], {
      tool: 'bcheck_import',
      params: { script: bcheckScript, enabled: false }
    });
    assert.deepEqual(bcheckResponse.result.content, [{
      type: 'text',
      text: JSON.stringify({ imported: 'bcheck_import' }, null, 2)
    }]);
  } finally {
    const childClosed = bridge.exitCode === null && bridge.signalCode === null
      ? once(bridge, 'close')
      : Promise.resolve();
    bridge.stdin.end();
    bridge.kill();
    await close(server);
    await childClosed.catch(() => {});
  }
});

test('returns backend error payloads as MCP tool errors', async () => {
  const port = await getFreePort();
  const bridgePath = path.join(__dirname, '..', 'mcp-bridge.js');
  const server = http.createServer((request, response) => {
    if (request.method === 'GET' && request.url === '/tools') {
      response.writeHead(200, { 'Content-Type': 'application/json' });
      response.end(JSON.stringify(['backend_error', 'backend_http_failure']));
      return;
    }

    let body = '';
    request.on('data', chunk => { body += chunk; });
    request.on('end', () => {
      const { tool } = JSON.parse(body);
      if (tool === 'backend_error') {
        response.writeHead(200, { 'Content-Type': 'application/json' });
        response.end(JSON.stringify({ error: 'Bambda import rejected by Burp' }));
        return;
      }
      response.writeHead(500, { 'Content-Type': 'text/plain' });
      response.end('Burp worker failed while importing');
    });
  });
  const bridge = spawn(process.execPath, [bridgePath], {
    env: {
      ...process.env,
      BURP_MCP_HOST: '127.0.0.1',
      BURP_MCP_PORT: String(port),
      BURP_MCP_TOKEN: 'test-token'
    },
    stdio: ['pipe', 'pipe', 'pipe']
  });
  const stdout = new LineReader(bridge.stdout);

  try {
    await new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(port, '127.0.0.1', resolve);
    });

    for (const [id, name] of [[1, 'burp_backend_error'], [2, 'burp_backend_http_failure']]) {
      bridge.stdin.write(JSON.stringify({
        jsonrpc: '2.0',
        id,
        method: 'tools/call',
        params: { name, arguments: {} }
      }) + '\n');
      const response = JSON.parse(await stdout.next());
      assert.equal(response.id, id);
      assert.equal(response.error, undefined);
      assert.equal(response.result.isError, true);
      assert.equal(response.result.content[0].type, 'text');
      if (id === 1) {
        assert.match(response.result.content[0].text, /Bambda import rejected by Burp/);
      } else {
        assert.match(response.result.content[0].text, /HTTP 500/);
        assert.match(response.result.content[0].text, /Burp worker failed while importing/);
      }
    }
  } finally {
    const childClosed = bridge.exitCode === null && bridge.signalCode === null
      ? once(bridge, 'close')
      : Promise.resolve();
    bridge.stdin.end();
    bridge.kill();
    await close(server);
    await childClosed.catch(() => {});
  }
});
