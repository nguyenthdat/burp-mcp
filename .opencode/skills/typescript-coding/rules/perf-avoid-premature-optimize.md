# perf-avoid-premature-optimize

> Profile before optimizing

## Why It Matters

Optimizing code based on intuition instead of measurement wastes engineering time on the wrong bottleneck, adds complexity (manual memoization, hand-unrolled loops, micro-tuned data structures) that makes the code harder to maintain, and sometimes makes performance *worse* because modern JS engines optimize idiomatic patterns better than "clever" ones. A profiler tells you where time and memory actually go; without one, "optimization" is usually just guessing, and the guess is wrong more often than engineers expect.

## Bad

```typescript
// Rewritten from a clear `.filter().map()` chain into a manually fused
// loop "for performance" — with no measurement showing it was ever slow.
function activeUserNames(users: User[]): string[] {
  const result: string[] = [];
  for (let i = 0; i < users.length; i++) {
    const u = users[i];
    if (u.status === "active") {
      result.push(u.name);
    }
  }
  return result;
}
// Saved maybe a few microseconds on a 50-item array, at the cost of
// readability, while the actual page load bottleneck (an unindexed
// database query) was never profiled or addressed.
```

## Good

```typescript
// Clear, idiomatic code first.
function activeUserNames(users: User[]): string[] {
  return users.filter((u) => u.status === "active").map((u) => u.name);
}

// Then measure with a profiler (e.g. `node --prof`, Chrome DevTools
// Performance tab, or `console.time`) to find the *actual* hot path
// before touching anything:
console.time("loadDashboard");
await loadDashboard(userId);
console.timeEnd("loadDashboard");
```

## A Practical Workflow

1. **Establish a baseline.** Use `console.time`/`performance.now()`, Chrome DevTools' Performance panel, `clinic.js`, or `node --prof` + `node --prof-process` to find where time is actually spent.
2. **Confirm the bottleneck is worth fixing.** A function called once per page load that takes 2ms is not worth optimizing even if you could halve it; a query running per-row in a loop over 10,000 rows is.
3. **Optimize the measured bottleneck**, and re-profile afterward to confirm the change actually helped — engine JIT behavior is often counter-intuitive.
4. **Add a regression benchmark** (e.g. with `tinybench` or `vitest bench`) for hot paths so future changes don't silently reintroduce the slowdown.

## See Also

- [perf-bundle-size-audit](perf-bundle-size-audit.md) - Audit bundle size and dependency weight regularly
- [perf-memoize-expensive](perf-memoize-expensive.md) - Memoize expensive pure computations
- [perf-avoid-unnecessary-allocation](perf-avoid-unnecessary-allocation.md) - Avoid allocating objects/arrays inside hot loops
