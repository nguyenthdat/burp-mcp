# async-abort-controller

> Use `AbortController` to make async operations cancellable

## Why It Matters

Without a cancellation mechanism, async work that's no longer needed — a `fetch` for a component that unmounted, a search request superseded by a newer keystroke — keeps running, wastes resources, and can produce stale results that clobber newer ones. `AbortController` is the standard, framework-agnostic way to propagate cancellation through `fetch`, streams, and any custom async function that accepts an `AbortSignal`. Abandoned promises still "leak" without it: they hold onto closures, timers, and network sockets until they eventually resolve on their own.

## Bad

```typescript
async function search(query: string, resultsEl: HTMLElement) {
  const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
  const results = await response.json();
  // If the user typed again before this resolved, this may overwrite
  // newer results with a stale response — there is no way to cancel it.
  renderResults(resultsEl, results);
}
```

## Good

```typescript
let currentSearch: AbortController | null = null;

async function search(query: string, resultsEl: HTMLElement): Promise<void> {
  currentSearch?.abort();
  const controller = new AbortController();
  currentSearch = controller;

  try {
    const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`, {
      signal: controller.signal,
    });
    const results = await response.json();
    renderResults(resultsEl, results);
  } catch (err) {
    if (err instanceof DOMException && err.name === "AbortError") {
      return; // superseded by a newer search, not a real failure
    }
    throw err;
  }
}
```

## Propagating AbortSignal Into Your Own Async Functions

```typescript
async function delay(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) return reject(signal.reason);
    const timer = setTimeout(resolve, ms);
    signal?.addEventListener("abort", () => {
      clearTimeout(timer);
      reject(signal.reason);
    });
  });
}
```

## Common Sources of an AbortSignal

- `AbortController#signal` — created and controlled manually, as above.
- `AbortSignal.timeout(ms)` — a signal that auto-aborts after a duration (see async-timeout-race).
- Framework lifecycle hooks (e.g. React `useEffect` cleanup) that create a controller per effect run and abort it on cleanup.

## See Also

- [async-timeout-race](async-timeout-race.md) - Implement timeouts by racing a promise against a timer
- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise floating; await, return, or explicitly void it
- [node-graceful-shutdown](node-graceful-shutdown.md) - Use cancellation signals to shut down in-flight work cleanly
