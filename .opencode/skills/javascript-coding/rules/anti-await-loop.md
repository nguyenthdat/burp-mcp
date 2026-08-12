# anti-await-loop

> Don't use `await` inside `forEach()` or `.map()` — use `for...of` or `Promise.all()`

## Why It Matters

`forEach()` doesn't wait for promises. Using `await` inside `forEach` creates floating promises that run concurrently but can't be tracked. `.map()` with `await` also creates an array of promises that aren't awaited — you need `Promise.all()` to wait for all of them. This anti-pattern creates silent bugs where operations appear complete but are still running.

## Bad

```js
// await in forEach — doesn't wait!
items.forEach(async (item) => {
  await save(item);  // Each save runs, but forEach doesn't wait for them
});
console.log('Done');  // Logs immediately — saves still running!

// await in map — fire and forget
const results = items.map(async (item) => {
  return await process(item);  // Returns array of promises, not results
});
// results is [Promise, Promise, ...] — not what you expected
```

## Good

```js
// Sequential — for...of with await
for (const item of items) {
  await save(item);
}
console.log('Done');  // Logs after all saves complete

// Parallel — Promise.all with map
const results = await Promise.all(
  items.map(item => process(item)),
);

// Concurrent with limited parallelism
const results = await Array.fromAsync(items, process);
```

## When Exceptions Apply

If you genuinely want fire-and-forget (e.g., non-critical analytics), make it explicit:

```js
items.forEach((item) => {
  analytics.track(item).catch(() => {});  // Explicitly fire-and-forget
});
```

## See Also

- [async-parallel-over-sequential](./async-parallel-over-sequential.md) - Parallel execution
- [async-array-from-async](./async-array-from-async.md) - Array.fromAsync
