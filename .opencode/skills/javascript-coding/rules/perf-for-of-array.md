# perf-for-of-array

> Prefer `for...of` over `forEach()` for arrays when performance matters

## Why It Matters

`forEach()` creates a function call per iteration, adding call overhead. `for...of` iterates directly without function calls, making it faster for large arrays. `forEach()` also can't be broken out of early (no `break`/`return`) and doesn't work with `await`. For simple iteration where you don't need the declarative style of `map`/`filter`/`reduce`, `for...of` is the most performant.

## Bad

```js
// forEach — function call overhead per element
items.forEach((item, index) => {
  process(item, index);
});

// Cannot break or return early
items.forEach(item => {
  if (item.isDone) return;  // Only returns from this callback, not forEach
  process(item);
});

// Cannot use await
items.forEach(async (item) => {
  await save(item);  // Each save runs concurrently — not sequential
});
```

## Good

```js
// for...of — no function call overhead, supports break/continue/await
for (const item of items) {
  process(item);
}

// Can break early
for (const item of items) {
  if (item.isDone) break;
  process(item);
}

// Correct sequential async
for (const item of items) {
  await save(item);  // Sequential — each await waits for the previous
}
```

## Performance Comparison

```js
// Fastest → Slowest for array iteration:
// 1. for (let i = 0; i < arr.length; i++)  — classic loop
// 2. for...of                                — clean, fast
// 3. forEach()                               — function call overhead
// 4. map/filter/reduce                       — creates new arrays

// Choose clarity over micro-optimizations unless profiling shows a hotspot
```

## When Exceptions Apply

For arrays smaller than 100 elements, the performance difference is negligible. Use the most readable option. For massive arrays in hot paths, `for...of` is preferred.

## See Also

- [fn-map-over-for](./fn-map-over-for.md) - map/filter/reduce for transformations
- [anti-await-loop](./anti-await-loop.md) - Don't await inside forEach
