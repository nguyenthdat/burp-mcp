# node-graceful-shutdown

> Handle `SIGTERM`/`SIGINT` for graceful shutdown

## Why It Matters

Orchestrators like Kubernetes and process managers like systemd send `SIGTERM` and wait a grace period before forcibly killing a process with `SIGKILL`. If your app doesn't handle `SIGTERM`, in-flight HTTP requests get dropped, database transactions get cut off mid-write, and messages pulled from a queue are lost because they were never acknowledged. Graceful shutdown means: stop accepting new work, let in-flight work finish (or time out), close resources (DB pools, message consumers) cleanly, and only then exit — turning deploys and autoscaling events from a source of dropped requests into invisible, zero-downtime operations.

## Bad

```typescript
import http from 'node:http';

const server = http.createServer((req, res) => {
  res.end('ok');
});

server.listen(3000);
// No signal handling: SIGTERM kills the process immediately,
// dropping any request that's mid-flight and leaving DB connections open.
```

## Good

```typescript
import http from 'node:http';
import { setTimeout as delay } from 'node:timers/promises';

const server = http.createServer((req, res) => {
  res.end('ok');
});

server.listen(3000);

let shuttingDown = false;

async function shutdown(signal: string) {
  if (shuttingDown) return;
  shuttingDown = true;
  console.log(`Received ${signal}, shutting down gracefully`);

  // Stop accepting new connections; wait for in-flight requests to drain.
  const closeServer = new Promise<void>((resolve, reject) => {
    server.close((err) => (err ? reject(err) : resolve()));
  });

  try {
    await Promise.race([closeServer, delay(10_000)]); // hard cap: 10s
    await closeDatabasePool();
    await closeMessageQueueConsumer();
    console.log('Shutdown complete');
    process.exit(0);
  } catch (err) {
    console.error('Error during shutdown', err);
    process.exit(1);
  }
}

process.on('SIGTERM', () => void shutdown('SIGTERM'));
process.on('SIGINT', () => void shutdown('SIGINT'));

async function closeDatabasePool() {
  /* pool.end() etc. */
}
async function closeMessageQueueConsumer() {
  /* consumer.close() etc. */
}
```

## Checklist for a Real Shutdown Handler

| Step | Why |
|---|---|
| Stop the HTTP server from accepting new sockets (`server.close`) | Prevents new work arriving mid-shutdown |
| Let existing requests finish, with a timeout | Balances zero-downtime vs. hung shutdowns |
| Close DB pools / Redis clients / message consumers | Prevents connection leaks and unacked messages |
| Set a hard deadline (`SIGKILL` fallback) | Orchestrators kill anyway after ~30s; exit before that |
| Guard against double-invocation | `SIGTERM` and `SIGINT` can both fire during a deploy |

## See Also

- [node-process-exit-avoid](node-process-exit-avoid.md) - Avoid `process.exit()` in library code; let the caller control the process
- [async-abort-controller](async-abort-controller.md) - Use `AbortController` to cancel in-flight async work
- [err-finally-cleanup](err-finally-cleanup.md) - Use `finally` to guarantee cleanup runs
