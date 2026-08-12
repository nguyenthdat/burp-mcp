# perf-debounce-throttle

> Debounce or throttle high-frequency event handlers

## Why It Matters

Events like `scroll`, `resize`, `mousemove`, and keystroke-driven search inputs can fire dozens or hundreds of times per second; if the handler does expensive work (a network request, a layout recalculation, a large re-render) on every single event, the browser's main thread gets flooded with redundant work, causing visible jank or wasted API quota. Debouncing (wait for a pause in events before acting) and throttling (act at most once per interval) both reduce the handler's call frequency to something proportional to what the user actually needs.

## Bad

```typescript
// Fires a network request on every single keystroke
searchInput.addEventListener("input", async (e) => {
  const results = await fetchSearchResults((e.target as HTMLInputElement).value);
  renderResults(results);
});

// Runs expensive layout work on every scroll event, dozens of times per second
window.addEventListener("scroll", () => {
  updateParallaxPositions();
});
```

## Good

```typescript
function debounce<Args extends unknown[]>(fn: (...args: Args) => void, delayMs: number) {
  let timeoutId: ReturnType<typeof setTimeout> | undefined;
  return (...args: Args) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delayMs);
  };
}

function throttle<Args extends unknown[]>(fn: (...args: Args) => void, intervalMs: number) {
  let lastRun = 0;
  return (...args: Args) => {
    const now = Date.now();
    if (now - lastRun >= intervalMs) {
      lastRun = now;
      fn(...args);
    }
  };
}

const debouncedSearch = debounce(async (query: string) => {
  renderResults(await fetchSearchResults(query));
}, 300);
searchInput.addEventListener("input", (e) => debouncedSearch((e.target as HTMLInputElement).value));

const throttledParallax = throttle(updateParallaxPositions, 16); // ~60fps
window.addEventListener("scroll", throttledParallax);
```

## When To Use Which

| Pattern | Behavior | Good for |
|---|---|---|
| Debounce | Waits for a pause in events, then runs once | Search-as-you-type, form validation, window resize settling |
| Throttle | Runs at most once per fixed interval, even during continuous events | Scroll-linked animation, mousemove drag tracking, analytics ping |

Prefer a well-tested implementation (`lodash.debounce`, `lodash.throttle`) over hand-rolling one in production code — edge cases around `this` binding, leading/trailing edge execution, and cancellation are easy to get subtly wrong.

## See Also

- [test-fake-timers](test-fake-timers.md) - Use fake timers to test time-dependent code deterministically
- [perf-avoid-blocking-event-loop](perf-avoid-blocking-event-loop.md) - Avoid long synchronous operations that block the event loop
- [async-abort-controller](async-abort-controller.md) - cancel in-flight requests made obsolete by newer input
