# async-concurrency-limit

> Bound concurrency with a limiter when processing large batches

## Why It Matters

`Promise.all` over a large array starts every operation at once, which for thousands of items can exhaust file descriptors, hit downstream rate limits, or overwhelm a database connection pool. A concurrency limiter caps how many operations run simultaneously while still processing the whole batch far faster than a fully sequential loop. Choosing the right limit is a deliberate trade-off between throughput and not tripping the failure modes of whatever system you're calling.

## Bad

```typescript
async function importAllUsers(records: UserRecord[]): Promise<void> {
  // 50,000 concurrent DB writes at once — pool exhaustion, timeouts, crashes
  await Promise.all(records.map((r) => db.insertUser(r)));
}
```

## Good

```typescript
import pLimit from "p-limit";

async function importAllUsers(records: UserRecord[]): Promise<void> {
  const limit = pLimit(10); // at most 10 concurrent writes
  await Promise.all(records.map((r) => limit(() => db.insertUser(r))));
}
```

## Implementing a Minimal Limiter Without a Dependency

```typescript
async function mapWithConcurrency<T, R>(
  items: T[],
  limit: number,
  fn: (item: T) => Promise<R>,
): Promise<R[]> {
  const results: R[] = new Array(items.length);
  let nextIndex = 0;

  async function worker() {
    while (nextIndex < items.length) {
      const i = nextIndex++;
      results[i] = await fn(items[i]);
    }
  }

  await Promise.all(Array.from({ length: limit }, worker));
  return results;
}

// Usage
const users = await mapWithConcurrency(ids, 10, fetchUser);
```

## Choosing a Limit

| Constraint | Typical starting point |
|---|---|
| External HTTP API with a published rate limit | Match the API's documented concurrent-request allowance |
| Database connection pool of size N | N minus headroom for other queries (e.g. N - 2) |
| CPU-bound work via worker threads | `os.cpus().length` |
| Unknown/third-party service | Start conservative (5-10), measure, adjust |

`p-limit` is the de facto standard library for this in Node/TypeScript; it's tiny, well-typed, and composes cleanly with `Promise.all`.

## See Also

- [async-avoid-sequential-await](async-avoid-sequential-await.md) - Avoid awaiting independent operations sequentially inside loops
- [async-promise-all-parallel](async-promise-all-parallel.md) - Use `Promise.all` to run independent async work concurrently
- [node-worker-threads-cpu](node-worker-threads-cpu.md) - Offload CPU-bound work to worker threads
