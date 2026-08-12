# async-void-operator

> Use the `void` operator to mark an intentionally ignored promise

## Why It Matters

Sometimes you genuinely want to start an async operation without waiting for it — logging, analytics pings, cache warming — but leaving a bare promise-producing expression statement is indistinguishable, to a reader or to `@typescript-eslint/no-floating-promises`, from a forgotten `await`. Prefixing the call with `void` documents the intent explicitly and satisfies the lint rule, while still letting the promise run to completion (or rejection) in the background.

## Bad

```typescript
function trackEvent(name: string, props: Record<string, unknown>) {
  // Is this a bug (forgot await) or intentional fire-and-forget?
  // A reader — and a linter — can't tell.
  analytics.send(name, props);
}
```

## Good

```typescript
function trackEvent(name: string, props: Record<string, unknown>) {
  // Explicitly fire-and-forget; failures are handled inside .catch
  void analytics.send(name, props).catch((err) => {
    logger.warn("analytics send failed", { err, name });
  });
}
```

## `void` Does Not Suppress Unhandled Rejections

```typescript
// void only tells the linter/reader "I'm not using this value" —
// it does NOT attach a rejection handler. Always pair void with .catch
// (or ensure the callee never rejects) to avoid unhandled rejections.

void doSomethingAsync(); // BAD if doSomethingAsync can reject
void doSomethingAsync().catch(reportError); // GOOD
```

## `void` vs Ignoring the Return Value Silently

| Pattern | Linter-visible intent | Handles rejection |
|---|---|---|
| `promiseFn();` | No — indistinguishable from a bug | No |
| `void promiseFn();` | Yes — explicit fire-and-forget | No |
| `void promiseFn().catch(handler);` | Yes | Yes |
| `await promiseFn();` | Yes — explicit wait | Yes (via surrounding try/catch) |

## See Also

- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise floating; await, return, or explicitly void it
- [err-unhandled-rejection](err-unhandled-rejection.md) - Handling process-level unhandled promise rejections
- [lint-no-floating-promises-rule](lint-no-floating-promises-rule.md) - Configuring the ESLint rule that enforces this
