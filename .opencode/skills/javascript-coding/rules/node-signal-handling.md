# node-signal-handling

> Handle SIGTERM and SIGINT for graceful shutdown — close connections, finish requests, clean up resources

## Why It Matters

A process that exits immediately on SIGTERM drops in-flight requests, leaves database connections open, and loses buffered data. Graceful shutdown gives the process time to finish current work before exiting. Container orchestrators (Kubernetes, Docker) send SIGTERM and expect a graceful exit within a timeout (typically 30s).

## Bad

```js
// No signal handling — process terminates immediately on SIGTERM
import { createServer } from 'node:http';

const server = createServer(async (req, res) => {
  await processRequest(req);  // May be killed mid-request
  res.end('OK');
});

server.listen(3000);
```

## Good

```js
import { createServer } from 'node:http';

let server;
let isShuttingDown = false;

function shutdown(signal) {
  if (isShuttingDown) return;
  isShuttingDown = true;

  console.log(`${signal} received — shutting down gracefully`);

  // Stop accepting new connections
  server.close(() => {
    console.log('Server closed');
  });

  // Close database connections
  db.disconnect().then(() => console.log('Database disconnected'));

  // Force exit after timeout
  setTimeout(() => {
    console.error('Forced shutdown after timeout');
    process.exit(1);
  }, 10000).unref();
}

server = createServer(async (req, res) => {
  if (isShuttingDown) {
    res.writeHead(503, { 'Connection': 'close' });
    return res.end('Server is shutting down');
  }
  await processRequest(req);
  res.end('OK');
});

process.on('SIGTERM', () => shutdown('SIGTERM'));
process.on('SIGINT', () => shutdown('SIGINT'));

server.listen(3000);
```

## When Exceptions Apply

Short-lived CLI tools and one-shot scripts don't need graceful shutdown. For long-running servers, it's essential.

## See Also

- [err-global-handlers](./err-global-handlers.md) - Global error handlers
- [node-process-exit](./node-process-exit.md) - Exit code conventions
