# err-finally-cleanup

> Use `finally` for cleanup that must run regardless of outcome

## Why It Matters

Cleanup logic duplicated in both the success path and every catch branch is easy to miss on one of them — add a new early `return` or a new thrown error later, and the resource (a file handle, a database connection, a lock, a loading spinner) leaks because the duplicated cleanup call wasn't copied to the new exit path. `finally` runs exactly once, on every exit from the `try` block — success, thrown error, or early `return` — so the cleanup logic only has to be written once and can't be forgotten as the function evolves.

## Bad

```typescript
async function withLock<T>(lockId: string, fn: () => Promise<T>): Promise<T> {
  await acquireLock(lockId);
  try {
    const result = await fn();
    await releaseLock(lockId); // duplicated...
    return result;
  } catch (err) {
    await releaseLock(lockId); // ...and easy to miss adding here when refactoring
    throw err;
  }
}
```

## Good

```typescript
async function withLock<T>(lockId: string, fn: () => Promise<T>): Promise<T> {
  await acquireLock(lockId);
  try {
    return await fn();
  } finally {
    await releaseLock(lockId); // runs on success, on thrown error, and on early return
  }
}
```

## Common Cleanup Targets

```typescript
function readConfigFile(path: string): Config {
  const fd = fs.openSync(path, "r");
  try {
    const raw = fs.readFileSync(fd, "utf-8");
    return JSON.parse(raw);
  } finally {
    fs.closeSync(fd); // always closes, even if JSON.parse throws
  }
}

function withSpinner<T>(fn: () => T): T {
  spinner.start();
  try {
    return fn();
  } finally {
    spinner.stop(); // stops even if fn() throws
  }
}
```

## Pitfall: Returning or Throwing Inside `finally`

A `return` or `throw` inside a `finally` block silently overrides whatever the `try` or `catch` block was about to produce, including swallowing an in-flight exception. Avoid control-flow statements in `finally` — it should only perform side effects (cleanup), never influence the return value or error.

```typescript
function risky(): number {
  try {
    throw new Error("boom");
  } finally {
    return 42; // swallows the Error entirely — caller never sees it. Avoid this.
  }
}
```

## See Also

- [err-rethrow-context](err-rethrow-context.md) - Add context when rethrowing instead of losing the original error
- [async-abort-controller](async-abort-controller.md) - Use AbortController to cancel in-flight async work
- [node-graceful-shutdown](node-graceful-shutdown.md) - Handle process signals for graceful shutdown
