# err-unhandled-rejection

> Register process-level handlers for unhandled promise rejections

## Why It Matters

A rejected promise with no `.catch` and no surrounding `try/catch` doesn't just disappear — in Node.js it fires an `unhandledRejection` event on `process`, and since Node 15 the default behavior is to crash the process entirely. Without a registered handler, you either get an ungraceful crash with a generic stack trace, or (in older configurations) a silently swallowed rejection that leaves the application in an inconsistent state with zero record of what happened. A process-level handler is the last line of defense that ensures these failures are always logged before the process exits.

## Bad

```typescript
// No global handler registered anywhere in the app
function backgroundCleanup() {
  cleanupTempFiles(); // returns a promise, not awaited, not .catch'd
}

backgroundCleanup();
// If cleanupTempFiles() rejects, Node prints a generic warning (or crashes)
// with no application-specific context about what was running or why
```

## Good

```typescript
process.on("unhandledRejection", (reason, promise) => {
  logger.error("unhandled promise rejection", {
    reason: reason instanceof Error ? reason.stack : reason,
  });
  // Treat as fatal: an unhandled rejection means a bug slipped past every layer of
  // handling in the app. Exit so the process supervisor (systemd, k8s, pm2) restarts
  // it cleanly, rather than continuing to run in a possibly-corrupted state.
  process.exit(1);
});

process.on("uncaughtException", (err) => {
  logger.error("uncaught exception", { error: err.stack });
  process.exit(1);
});

function backgroundCleanup() {
  cleanupTempFiles().catch((err) => {
    logger.error("cleanup failed", { error: err }); // handled at the source too — the process
    // handler above is a safety net, not the primary mechanism
  });
}
```

## Defense in Depth, Not a Substitute

Process-level handlers exist to catch what *slips through*, not to replace per-call error handling. Every promise you create should still have an explicit `.catch` or be `await`ed inside a `try/catch` — see `async-no-floating-promises`. Rely on the global handler only as the last safety net for genuine bugs.

## Browser Equivalent

```typescript
window.addEventListener("unhandledrejection", (event) => {
  console.error("unhandled rejection:", event.reason);
  reportToMonitoring(event.reason);
  event.preventDefault(); // suppress the default console error if you've already reported it
});
```

## See Also

- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise unawaited and unhandled
- [err-never-swallow](err-never-swallow.md) - Never silently swallow errors in empty catch blocks
- [node-graceful-shutdown](node-graceful-shutdown.md) - Handle process signals for graceful shutdown
