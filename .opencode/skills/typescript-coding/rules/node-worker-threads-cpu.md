# node-worker-threads-cpu

> Use `worker_threads` for CPU-bound work in a Node.js server

## Why It Matters

Node's concurrency model is built around a single-threaded event loop with an async I/O pool — it's excellent at handling many concurrent network requests, but a CPU-bound task (image resizing, large JSON transforms, cryptographic hashing, complex regex) occupies that one thread and blocks every other request until it finishes. `worker_threads` runs JavaScript on a genuinely separate thread with its own event loop, so CPU-heavy work no longer competes with request handling. This is different from `child_process`, which spawns a separate OS process with higher overhead and no shared memory; workers are lighter-weight and can share memory via `SharedArrayBuffer` when needed.

## Bad

```typescript
import http from 'node:http';
import crypto from 'node:crypto';

const server = http.createServer((req, res) => {
  // Blocks the event loop for every other in-flight request
  const hash = crypto.pbkdf2Sync('password', 'salt', 500_000, 64, 'sha512');
  res.end(hash.toString('hex'));
});

server.listen(3000);
```

## Good

```typescript
// worker.ts
import { parentPort, workerData } from 'node:worker_threads';
import crypto from 'node:crypto';

const hash = crypto.pbkdf2Sync(workerData.password, 'salt', 500_000, 64, 'sha512');
parentPort?.postMessage(hash.toString('hex'));
```

```typescript
// server.ts
import http from 'node:http';
import { Worker } from 'node:worker_threads';

function hashInWorker(password: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const worker = new Worker('./worker.js', { workerData: { password } });
    worker.once('message', resolve);
    worker.once('error', reject);
    worker.once('exit', (code) => {
      if (code !== 0) reject(new Error(`Worker stopped with exit code ${code}`));
    });
  });
}

const server = http.createServer(async (req, res) => {
  const hash = await hashInWorker('password'); // event loop stays free
  res.end(hash);
});

server.listen(3000);
```

## Worker Pools

Spawning a new `Worker` per request has real overhead (thread startup, module re-evaluation). For sustained load, use a pool that reuses a fixed set of long-lived workers, such as `piscina` or Node's built-in `worker_threads` combined with a task queue:

```bash
npm install piscina
```

```typescript
import Piscina from 'piscina';

const pool = new Piscina({ filename: new URL('./worker.js', import.meta.url).href });
const hash = await pool.run({ password: 'password' });
```

## See Also

- [node-avoid-sync-fs](node-avoid-sync-fs.md) - Avoid synchronous `fs` calls on a server's request path
- [perf-avoid-blocking-event-loop](perf-avoid-blocking-event-loop.md) - Avoid long synchronous work that blocks the event loop
- [perf-worker-offload](perf-worker-offload.md) - Offload expensive work to workers instead of the main thread
