# async-parallel-over-sequential

> Use Promise.all() or concurrent mapping over sequential awaits in a loop

## Why It Matters

Awaiting promises one-by-one in a loop serializes independent operations, needlessly multiplying total latency. When operations are independent, run them concurrently with `Promise.all()` or a concurrency-limited mapper to minimize wall-clock time.

## Bad

```js
// Sequential — each fetch waits for the previous to complete
async function getUserProfiles(userIds) {
  const profiles = [];
  for (const id of userIds) {
    const profile = await fetchUser(id);  // Blocks serial
    profiles.push(profile);
  }
  return profiles;
}
// Total time: sum of all individual latencies
```

## Good

```js
// Parallel — all fetches run concurrently
async function getUserProfiles(userIds) {
  const promises = userIds.map(id => fetchUser(id));
  return await Promise.all(promises);
}
// Total time: max of individual latencies

// With concurrency limit for large arrays
async function getUserProfilesBatched(userIds, concurrency = 5) {
  const results = [];
  for (let i = 0; i < userIds.length; i += concurrency) {
    const batch = userIds.slice(i, i + concurrency).map(id => fetchUser(id));
    results.push(...(await Promise.all(batch)));
  }
  return results;
}
```

## When Exceptions Apply

Sequential awaits are correct when each operation depends on the result of the previous one, or when you need to respect rate limits by processing one at a time:

```js
async function processChain(initial) {
  const step1 = await transformA(initial);
  const step2 = await transformB(step1);  // Depends on step1
  return await transformC(step2);          // Depends on step2
}
```

## See Also

- [async-promise-allsettled](./async-promise-allsettled.md) - Use allSettled when partial failures are ok
- [anti-await-loop](./anti-await-loop.md) - Don't await inside forEach
