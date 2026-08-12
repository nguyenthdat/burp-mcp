# test-fake-timers

> Use fake timers to test time-dependent code deterministically

## Why It Matters

Tests that call `setTimeout`, wait on real debounce/throttle windows, or depend on wall-clock time are slow and flaky: a CI runner under load can make a "wait 50ms" test miss its window, and a test suite with dozens of real waits adds minutes to every run. Fake timers let you advance virtual time instantly and deterministically, so a rate limiter with a 24-hour window or a retry with exponential backoff can be fully tested in milliseconds, with no race conditions.

## Bad

```typescript
import { expect, it } from "vitest";
import { debounce } from "./debounce";

it("should call the function once after the delay", async () => {
  const fn = vi.fn();
  const debounced = debounce(fn, 300);

  debounced();
  debounced();

  // Real wall-clock wait: slow, and flaky under CI load
  await new Promise((resolve) => setTimeout(resolve, 350));

  expect(fn).toHaveBeenCalledTimes(1);
});
```

## Good

```typescript
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { debounce } from "./debounce";

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

it("should call the function once after the delay", () => {
  const fn = vi.fn();
  const debounced = debounce(fn, 300);

  debounced();
  debounced();

  vi.advanceTimersByTime(300);

  expect(fn).toHaveBeenCalledTimes(1);
});
```

## Common Patterns

- `vi.advanceTimersByTime(ms)` fires timers synchronously up to the given offset; `vi.runAllTimers()` exhausts every pending timer, useful for retry loops that reschedule themselves.
- For code under test that also awaits real promises (e.g. `await Promise.resolve()` between timer ticks), use `await vi.advanceTimersByTimeAsync(ms)` so microtasks flush between ticks.
- Always restore real timers in `afterEach`/`afterAll` — leaked fake timers in one test file can hang or misbehave in unrelated tests that assume real time.
- Combine with mocking `Date.now()`/`vi.setSystemTime()` when the code under test reads the wall clock directly rather than only scheduling via `setTimeout`.

## See Also

- [test-mock-boundaries](test-mock-boundaries.md) - Mock external boundaries (network, filesystem, clock), not internal implementation details
- [perf-debounce-throttle](perf-debounce-throttle.md) - Debounce or throttle high-frequency event handlers
- [async-retry-backoff](async-retry-backoff.md) - retry with exponential backoff, a common thing to test with fake timers
