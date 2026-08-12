# perf-memoize-expensive

> Memoize expensive pure computations

## Why It Matters

Recomputing an expensive pure function every time it's called — a heavy parse, a recursive Fibonacci-style computation, a derived value recalculated on every render — wastes CPU cycles on work whose answer hasn't changed since the last call. Memoization caches the result keyed on the input, trading a small amount of memory for skipping redundant work; it's only correct for pure functions, since caching a value that depends on hidden mutable state will return stale, wrong answers.

## Bad

```typescript
// Recomputed from scratch on every call, even with the same input,
// and called on every keystroke in a search-as-you-type UI.
function expensiveScore(query: string, corpus: Document[]): number {
  return corpus.reduce((score, doc) => score + computeRelevance(query, doc), 0);
}
```

## Good

```typescript
const scoreCache = new Map<string, number>();

function expensiveScore(query: string, corpus: Document[]): number {
  const cacheKey = `${query}:${corpus.length}`;
  const cached = scoreCache.get(cacheKey);
  if (cached !== undefined) return cached;

  const score = corpus.reduce((total, doc) => total + computeRelevance(query, doc), 0);
  scoreCache.set(cacheKey, score);
  return score;
}
```

```typescript
// React: memoize a derived value across re-renders
const sortedItems = useMemo(() => [...items].sort(compareByPriority), [items]);
```

## Guidelines

- Only memoize pure functions — same input always produces the same output, with no reliance on external mutable state. Memoizing an impure function (e.g. one reading `Date.now()`) produces subtly wrong cached results.
- Bound the cache: use an LRU cache (e.g. `lru-cache` from npm) or a `WeakMap` keyed by object identity for cases where inputs are objects and you want the cache entry to be garbage-collected when the input is no longer referenced elsewhere.
- Choose the cache key carefully — memoizing on `JSON.stringify(args)` is simple but can be slower than the function itself for large inputs; prefer a cheap composite key when possible.
- In React, prefer `useMemo`/`useCallback` only for computations that are actually expensive or that need referential stability to avoid child re-renders — wrapping every value in `useMemo` adds overhead without benefit.
- Memoization trades memory for CPU; for functions called with a huge number of distinct inputs, an unbounded cache can become its own memory leak.

## See Also

- [perf-avoid-premature-optimize](perf-avoid-premature-optimize.md) - Profile before optimizing
- [fn-pure-functions](fn-pure-functions.md) - only pure functions are safe to memoize
- [perf-avoid-unnecessary-allocation](perf-avoid-unnecessary-allocation.md) - Avoid allocating objects/arrays inside hot loops
