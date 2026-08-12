# async-promise-all-parallel

> Use `Promise.all` to run independent async work concurrently

## Why It Matters

Awaiting independent promises one after another forces the total latency to be the sum of every individual operation, even though nothing about the operations requires that ordering. `Promise.all` starts every operation immediately and resolves once all of them finish, so the total latency becomes the duration of the slowest operation instead of the sum. For I/O-bound code — HTTP calls, database queries, file reads — this is often the single biggest, lowest-risk performance win available.

## Bad

```typescript
async function loadDashboard(userId: string) {
  // Sequential: ~300ms if each call takes ~100ms
  const user = await fetchUser(userId);
  const orders = await fetchOrders(userId);
  const notifications = await fetchNotifications(userId);

  return { user, orders, notifications };
}
```

## Good

```typescript
async function loadDashboard(userId: string) {
  // Concurrent: ~100ms total, all three start at once
  const [user, orders, notifications] = await Promise.all([
    fetchUser(userId),
    fetchOrders(userId),
    fetchNotifications(userId),
  ]);

  return { user, orders, notifications };
}
```

## Promise.all vs Promise.allSettled vs Promise.race

| API | Behavior on rejection | Use when |
|---|---|---|
| `Promise.all` | Rejects immediately with the first error | All results are required; any failure invalidates the batch |
| `Promise.allSettled` | Never rejects; each result is `{status, value \| reason}` | You want partial results even if some fail |
| `Promise.race` | Settles as soon as the first promise settles (fulfilled or rejected) | Implementing timeouts or "first responder wins" |
| `Promise.any` | Fulfills with the first fulfillment, rejects only if all reject | You want the first success and can ignore failures |

## Watch Out: This Only Helps Independent Work

```typescript
// Wrong use: b depends on a's result, Promise.all can't help here
const [a, b] = await Promise.all([fetchA(), fetchB(a)]); // ReferenceError: a is not defined here anyway

// Correct: sequential because of the real dependency
const a = await fetchA();
const b = await fetchB(a);
```

## See Also

- [async-avoid-sequential-await](async-avoid-sequential-await.md) - Avoid awaiting independent operations sequentially inside loops
- [err-promise-allsettled](err-promise-allsettled.md) - Use `Promise.allSettled` when partial failure is acceptable
- [async-concurrency-limit](async-concurrency-limit.md) - Bound concurrency with a limiter when processing large batches
