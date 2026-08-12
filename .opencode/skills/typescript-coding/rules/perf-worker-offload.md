# perf-worker-offload

> Offload CPU-heavy work to a worker thread instead of blocking the main thread

## Why It Matters

Node.js and browser JavaScript both run application code on a single main thread; a CPU-bound task (image processing, large data transformation, cryptographic hashing, complex report generation) run there blocks everything else — in Node, that means every other in-flight request stalls; in a browser, the UI freezes and input stops responding. Worker threads (`worker_threads` in Node, Web Workers in the browser) run on a separate thread, letting the main thread stay responsive while the heavy computation happens in parallel.

## Bad

```typescript
// Blocks the Node.js event loop for the entire duration of the hash —
// every other request queued behind this one has to wait.
import { pbkdf2Sync } from "node:crypto";

app.post("/register", (req, res) => {
  const hash = pbkdf2Sync(req.body.password, salt, 600_000, 64, "sha512"); // CPU-bound, synchronous
  saveUser(req.body.email, hash);
  res.sendStatus(201);
});
```

## Good

```typescript
// worker/hash-password.ts
import { parentPort, workerData } from "node:worker_threads";
import { pbkdf2Sync } from "node:crypto";

const { password, salt } = workerData;
const hash = pbkdf2Sync(password, salt, 600_000, 64, "sha512");
parentPort?.postMessage(hash);
```

```typescript
// server.ts
import { Worker } from "node:worker_threads";

function hashPasswordOffThread(password: string, salt: Buffer): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const worker = new Worker("./worker/hash-password.js", { workerData: { password, salt } });
    worker.once("message", resolve);
    worker.once("error", reject);
  });
}

app.post("/register", async (req, res) => {
  const hash = await hashPasswordOffThread(req.body.password, salt);
  await saveUser(req.body.email, hash);
  res.sendStatus(201);
});
```

## When To Reach For This

| Workload | Main thread | Worker thread |
|---|---|---|
| I/O-bound (DB query, HTTP call, file read) | Fine — Node's event loop already handles this efficiently via async I/O | Unnecessary overhead |
| CPU-bound, short (<1ms) | Fine | Overhead of spawning/messaging outweighs the benefit |
| CPU-bound, long (image resizing, crypto, parsing) | Blocks everything else | Correct fit |

- Use a worker pool (e.g. `piscina`) instead of spawning a new worker per request — worker startup has real overhead, and an unbounded pool can exhaust system resources under load.
- Data passed to/from a worker is copied (or transferred, for `ArrayBuffer`s) via structured clone — don't rely on passing complex objects with methods, class instances, or closures.
- For browser code, the equivalent is a Web Worker; use `Comlink` to simplify the message-passing boilerplate into something that feels like calling an async function directly.

## See Also

- [perf-avoid-blocking-event-loop](perf-avoid-blocking-event-loop.md) - Avoid long synchronous operations that block the event loop
- [perf-avoid-json-parse-large](perf-avoid-json-parse-large.md) - Avoid blocking synchronous JSON parsing of large payloads on hot paths
- [node-worker-threads-cpu](node-worker-threads-cpu.md) - Node-specific guidance on using worker_threads for CPU-bound work
