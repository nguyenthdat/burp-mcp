# async-timeout-race

> Implement timeouts by racing a promise against a timer

## Why It Matters

Without an explicit timeout, an async call that hangs — a stuck socket, a server that never responds — will keep your `await` suspended indefinitely, tying up resources and leaving users staring at a spinner forever. `Promise.race` between the real operation and a timer that rejects lets you bound the maximum wait time deterministically, independent of whether the underlying API supports cancellation itself. Combined with `AbortController`, you can both bound the wait *and* actually stop the underlying work instead of just abandoning it.

## Bad

```typescript
async function fetchWithNoLimit(url: string): Promise<Response> {
  // If the server never responds, this awaits forever
  return fetch(url);
}
```

## Good

```typescript
class TimeoutError extends Error {
  constructor(ms: number) {
    super(`Operation timed out after ${ms}ms`);
    this.name = "TimeoutError";
  }
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new TimeoutError(ms)), ms);
  });

  return Promise.race([promise, timeout]).finally(() => clearTimeout(timer));
}

async function fetchWithLimit(url: string): Promise<Response> {
  return withTimeout(fetch(url), 5000);
}
```

## Preferred: Combine With AbortSignal.timeout for fetch

```typescript
async function fetchWithLimit(url: string): Promise<Response> {
  // AbortSignal.timeout actually cancels the underlying request,
  // not just the wait — no dangling socket left behind.
  return fetch(url, { signal: AbortSignal.timeout(5000) });
}
```

## Why `Promise.race` Alone Is Not Enough

`Promise.race` only stops *waiting* for the loser; it does not cancel it. The original promise keeps running to completion in the background, still consuming memory and I/O. Always pair a race-based timeout with real cancellation (`AbortController`) when the underlying API supports it — reserve plain `Promise.race` timeouts for operations that have no cancellation hook at all.

## See Also

- [async-abort-controller](async-abort-controller.md) - Use `AbortController` to make async operations cancellable
- [async-retry-backoff](async-retry-backoff.md) - Retry transient failures with exponential backoff and jitter
- [err-custom-error-class](err-custom-error-class.md) - Define a custom `Error` subclass like `TimeoutError`
