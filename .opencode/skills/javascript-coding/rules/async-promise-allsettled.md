# async-promise-allsettled

> Use Promise.allSettled() when partial failures are acceptable

## Why It Matters

`Promise.all()` short-circuits on the first rejection, discarding results from other promises that may have succeeded. When you care about collecting both successes and failures (e.g., batch processing, health checks, multi-source data fetching), `Promise.allSettled()` reports every outcome.

## Bad

```js
// First rejection kills everything else
try {
  const results = await Promise.all(urls.map(url => fetch(url)));
} catch (err) {
  // All successful fetches are lost — we can only see the first error
  console.error('Batch failed:', err);
}
```

## Good

```js
const results = await Promise.allSettled(urls.map(url => fetch(url)));

const succeeded = results
  .filter(r => r.status === 'fulfilled')
  .map(r => r.value);

const failed = results
  .filter(r => r.status === 'rejected')
  .map(r => r.reason);

console.log(`Succeeded: ${succeeded.length}, Failed: ${failed.length}`);

// Report failed URLs
for (let i = 0; i < results.length; i++) {
  if (results[i].status === 'rejected') {
    console.error(`URL ${urls[i]} failed:`, results[i].reason);
  }
}
```

## When Exceptions Apply

Use `Promise.all()` when all operations must succeed for the overall operation to be valid — e.g., database transactions where partial success is wrong:

```js
// All must succeed — a partial commit would corrupt state
const [accounts, settings, profile] = await Promise.all([
  fetchAccounts(userId),
  fetchSettings(userId),
  fetchProfile(userId),
]);
```

## See Also

- [async-parallel-over-sequential](./async-parallel-over-sequential.md) - Concurrent promise handling
- [err-aggregate-error](./err-aggregate-error.md) - AggregateError for multiple errors
