# node-avoid-sync-fs

> Avoid synchronous `fs` calls on a server's request path

## Why It Matters

Node.js runs JavaScript on a single thread; a synchronous call like `fs.readFileSync` blocks that thread until the disk I/O completes. On a request handler, that means every other in-flight request — even ones with nothing to do with the file — is frozen for the duration of the read. Under load, a slow disk or a large file turns a single handler's blocking call into a latency spike for the entire process. Synchronous `fs` calls are fine for one-time startup work (reading a config file before the server starts listening) but are a correctness bug on any code path a request can reach.

## Bad

```typescript
import fs from 'node:fs';
import http from 'node:http';

const server = http.createServer((req, res) => {
  // Blocks the event loop for every concurrent request while this runs
  const html = fs.readFileSync('./views/index.html', 'utf8');
  res.end(html);
});

server.listen(3000);
```

## Good

```typescript
import fs from 'node:fs/promises';
import http from 'node:http';

const server = http.createServer(async (req, res) => {
  try {
    const html = await fs.readFile('./views/index.html', 'utf8');
    res.end(html);
  } catch (err) {
    res.statusCode = 500;
    res.end('Internal Server Error');
  }
});

server.listen(3000);
```

## When Sync Is Acceptable

Synchronous `fs` calls are fine outside the request path:

```typescript
// Startup-time config load, before server.listen() — no concurrent requests yet
const config = JSON.parse(fs.readFileSync('./config.json', 'utf8'));

// One-off CLI scripts that don't serve concurrent traffic
```

For frequently-read files on the request path, cache the parsed result in memory (with a file watcher to invalidate) instead of re-reading — sync or async — on every request.

## See Also

- [node-streams-backpressure](node-streams-backpressure.md) - Use streams with backpressure for large I/O instead of buffering in memory
- [perf-avoid-blocking-event-loop](perf-avoid-blocking-event-loop.md) - Avoid long synchronous work that blocks the event loop
- [node-worker-threads-cpu](node-worker-threads-cpu.md) - Use `worker_threads` for CPU-bound work in a Node.js server
