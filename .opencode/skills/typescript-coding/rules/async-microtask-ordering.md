# async-microtask-ordering

> Understand microtask vs. macrotask ordering to avoid subtle scheduling bugs

## Why It Matters

`await`, `.then()`, and `queueMicrotask` schedule callbacks on the **microtask queue**, which the JS runtime always drains completely before moving on to the next **macrotask** (a `setTimeout` callback, an I/O event, a UI render). Code that assumes a `setTimeout(fn, 0)` will run "at the same time" as a resolved promise callback, or that mixes microtask- and macrotask-scheduled work expecting FIFO ordering across the two queues, will observe an ordering that looks wrong and is hard to reproduce in ad hoc testing.

## Bad

```typescript
console.log("1: sync");

setTimeout(() => console.log("2: macrotask (setTimeout)"), 0);

Promise.resolve().then(() => console.log("3: microtask (promise)"));

console.log("4: sync");

// A developer expecting registration order (1, 2, 3, 4) will be
// surprised: the actual output is 1, 4, 3, 2 — all microtasks run
// before the next macrotask, regardless of the 0ms delay.
```

## Good

```typescript
// Be explicit about which queue you need, and don't rely on
// cross-queue interleaving assumptions in application logic.

async function afterCurrentSyncWorkAndMicrotasks() {
  await Promise.resolve(); // yields once to let queued microtasks run first
  doWork();
}

async function afterEventLoopTurn() {
  await new Promise((resolve) => setTimeout(resolve, 0)); // yields a full macrotask turn
  doWork(); // guaranteed to run after pending I/O callbacks, renders, etc.
}
```

## Ordering Reference

| Scheduled via | Queue | Drained |
|---|---|---|
| `await`, `.then()`, `.catch()`, `.finally()` | Microtask | Fully, before the next macrotask |
| `queueMicrotask(fn)` | Microtask | Fully, before the next macrotask |
| `setTimeout`, `setInterval` | Macrotask (timers) | One at a time, between microtask drains |
| `setImmediate` (Node.js only) | Check phase (after I/O, before timers of next loop) | One per event loop iteration |
| I/O callbacks, UI paint | Macrotask | One at a time, between microtask drains |

## Why This Matters for Tests

Test frameworks that use fake timers (`vi.useFakeTimers()`, Jest's `jest.useFakeTimers()`) advance macrotasks explicitly but still need microtasks to flush naturally — forgetting an extra `await Promise.resolve()` or `await vi.runAllTimersAsync()` after advancing fake timers is a common source of flaky async tests.

## See Also

- [test-fake-timers](test-fake-timers.md) - Testing code that depends on timers deterministically
- [async-top-level-await](async-top-level-await.md) - Use top-level `await` only at module entry points
- [perf-avoid-blocking-event-loop](perf-avoid-blocking-event-loop.md) - Keep the event loop responsive under load
