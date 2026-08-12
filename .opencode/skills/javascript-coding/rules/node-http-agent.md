# node-http-agent

> Reuse HTTP agents for connection pooling — don't create a new agent per request

## Why It Matters

Creating a new HTTP agent per request prevents connection reuse and keep-alive, causing a new TCP connection and TLS handshake for every request. This adds latency (hundreds of ms per request) and exhausts ephemeral ports under load. A shared agent pools connections and reuses them, dramatically improving throughput.

## Bad

```js
// New agent per request — no connection reuse
import { get } from 'node:https';

for (const url of urls) {
  get(url, (res) => {
    // Creates new TCP+TLS connection each time
  });
}

// fetch with no agent reuse
const response = await fetch(url);
```

## Good

```js
// Shared agent — connection pooling
import { Agent } from 'node:https';

const agent = new Agent({
  keepAlive: true,
  maxSockets: 50,
  maxFreeSockets: 10,
  timeout: 60000,
});

for (const url of urls) {
  get(url, { agent }, (res) => {
    // Reuses connections — much faster
  });
}

// fetch with undici dispatcher
import { Agent as FetchAgent } from 'undici';

const dispatcher = new FetchAgent({
  keepAliveTimeout: 10_000,
  keepAliveMaxTimeout: 60_000,
  connections: 50,
});

const response = await fetch(url, { dispatcher });
```

## When Exceptions Apply

For one-off requests in scripts, a default agent (auto-created) is fine. For servers making outbound requests, always use a shared agent.

## See Also

- [perf-object-pool](./perf-object-pool.md) - Object pooling pattern
- [async-parallel-over-sequential](./async-parallel-over-sequential.md) - Concurrent request handling
