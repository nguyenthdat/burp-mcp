# async-array-from-async

> Use Array.fromAsync() (ES2024) over manual async mapping

## Why It Matters

Converting an async iterable or mapping items asynchronously to an array requires manual loops or `Promise.all()` patterns. `Array.fromAsync()` provides a built-in, efficient way to collect async iterables and apply an async mapping function, with better ergonomics and edge-case handling.

## Bad

```js
// Manual async mapping — verbose
async function mapAsync(iterable, mapper) {
  const results = [];
  for (const item of iterable) {
    results.push(await mapper(item));
  }
  return results;
}

const users = await mapAsync(ids, fetchUser);

// From an async iterable
const items = [];
for await (const chunk of asyncIterable) {
  items.push(chunk);
}
```

## Good

```js
// Use Array.fromAsync with an async mapper
const users = await Array.fromAsync(ids, fetchUser);

// Sequential async mapping (not concurrent)
const users = await Array.fromAsync(ids, async (id) => {
  const response = await fetch(`/api/users/${id}`);
  return await response.json();
});

// From an async iterable
const items = await Array.fromAsync(asyncIterable);

// From a sync iterable with async mapper
const enriched = await Array.fromAsync(data, async (row) => {
  const details = await fetchDetails(row.id);
  return { ...row, ...details };
});
```

## Sync vs Async Mapping

```js
const ids = [1, 2, 3, 4, 5];

// Array.fromAsync runs mapper sequentially (good for rate limiting)
const sequential = await Array.fromAsync(ids, fetchUser);

// Promise.all runs mapper concurrently (good for independent requests)
const parallel = await Promise.all(ids.map(fetchUser));

// Choose based on use case, not API convenience
```

## When Exceptions Apply

If you need concurrent execution with a concurrency limit, `Array.fromAsync` won't help — it processes sequentially. Use a batching pattern with `Promise.all` instead.

## See Also

- [async-parallel-over-sequential](./async-parallel-over-sequential.md) - Choose concurrency strategy
- [fn-map-over-for](./fn-map-over-for.md) - Prefer map/filter/reduce
