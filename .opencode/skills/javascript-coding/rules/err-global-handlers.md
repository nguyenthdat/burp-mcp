# err-global-handlers

> Set process.on('uncaughtException') and 'unhandledRejection' handlers at startup

## Why It Matters

Unhandled promise rejections and uncaught exceptions crash the Node.js process. Without global handlers, your application terminates abruptly with no graceful shutdown, leaving in-flight requests incomplete and resources unreleased. Global handlers provide a last-resort safety net and enable graceful degradation.

## Bad

```js
// No global handlers — process crashes silently
async function startServer() {
  await initDB();
  app.listen(3000);
}

// Unhandled rejection somewhere in the app — boom
process.on('exit', (code) => {
  console.log(`Process exiting with code ${code}`);  // Too late
});
```

## Good

```js
import { createServer } from 'node:http';

let server;

process.on('uncaughtException', (err) => {
  console.error('Uncaught exception:', err);
  // Perform emergency cleanup then exit
  // Don't continue — the process is in an unknown state
  process.exit(1);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled rejection at:', promise, 'reason:', reason);
  // In Node.js 15+, unhandled rejections terminate the process
  // Log and let the process exit, or decide if it's safe to continue
});

process.on('SIGTERM', async () => {
  console.log('SIGTERM received — shutting down gracefully');
  server?.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
  // Force exit after 10s
  setTimeout(() => process.exit(1), 10000);
});

server = createServer(app).listen(3000);
```

## Graceful Shutdown Checklist

```js
process.on('SIGTERM', async () => {
  console.log('Starting graceful shutdown...');

  // 1. Stop accepting new requests
  server.close();

  // 2. Wait for in-flight requests to complete
  await waitForPendingRequests(5000);

  // 3. Close database connections
  await db.close();

  // 4. Close message queues
  await mq.close();

  console.log('Shutdown complete');
  process.exit(0);
});
```

## When Exceptions Apply

Global handlers should NOT be used for normal error recovery — use try/catch for expected errors. Global handlers are a last-resort safety net.

## See Also

- [err-avoid-silent-catch](./err-avoid-silent-catch.md) - Handle errors locally first
- [node-signal-handling](./node-signal-handling.md) - SIGTERM/SIGINT for graceful shutdown
