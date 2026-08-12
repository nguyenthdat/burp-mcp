# async-no-floating-promises

> Never leave a promise floating; await, return, or explicitly void it

## Why It Matters

A "floating" promise is one whose completion and rejection are never observed by the calling code. If it rejects, the error becomes an unhandled rejection — in Node.js this can crash the process, and in browsers it silently disappears into the console. Floating promises also break expected ordering: code after the call assumes the async work already happened, or a function returns before background work finishes, causing race conditions in tests and production alike.

## Bad

```typescript
function handleRequest(req: Request): Response {
  // Not awaited, not returned, error unhandled if auditLog rejects
  auditLog.record(req);
  return buildResponse(req);
}

async function onClick() {
  saveDraft(currentDocument); // fire-and-forget without acknowledging it
  navigateAway();
}
```

## Good

```typescript
async function handleRequest(req: Request): Promise<Response> {
  await auditLog.record(req);
  return buildResponse(req);
}

async function onClick() {
  // Explicit: we intend to not wait, but still handle failure
  void saveDraft(currentDocument).catch((err) => reportError("saveDraft failed", err));
  navigateAway();
}
```

## Enforcing This With a Lint Rule

```jsonc
// .eslintrc.json (requires @typescript-eslint)
{
  "rules": {
    "@typescript-eslint/no-floating-promises": "error",
    "@typescript-eslint/no-misused-promises": "error"
  }
}
```

`@typescript-eslint/no-floating-promises` flags any expression statement that produces a `Promise` and is neither awaited, returned, nor explicitly marked with `void`. This turns an easy-to-miss runtime bug into a compile-time/lint-time error.

## Three Legitimate Ways to Resolve a Promise-Producing Statement

| Intent | Pattern |
|---|---|
| Wait for it here | `await doWork();` |
| Delegate to caller | `return doWork();` |
| Intentionally fire-and-forget | `void doWork().catch(handleError);` |

## See Also

- [async-void-operator](async-void-operator.md) - Use the `void` operator to mark an intentionally ignored promise
- [err-unhandled-rejection](err-unhandled-rejection.md) - Handling process-level unhandled promise rejections
- [lint-no-floating-promises-rule](lint-no-floating-promises-rule.md) - Configuring the ESLint rule that enforces this
