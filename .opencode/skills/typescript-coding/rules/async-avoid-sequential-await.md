# async-avoid-sequential-await

> Avoid awaiting independent operations sequentially inside loops

## Why It Matters

`await` inside a `for` loop pauses the entire loop until that iteration's promise settles before starting the next one, even when the operations don't depend on each other. For N items each taking T milliseconds, this turns an operation that could complete in roughly T milliseconds into one that takes N × T milliseconds. This is one of the most common and costly performance bugs in real-world async TypeScript code, especially in scripts that call external APIs or databases in a loop.

## Bad

```typescript
async function enrichUsers(ids: string[]): Promise<User[]> {
  const users: User[] = [];
  for (const id of ids) {
    // Each fetch waits for the previous one to finish first
    const user = await fetchUser(id);
    users.push(user);
  }
  return users;
}
// 100 ids at 50ms each = 5000ms
```

## Good

```typescript
async function enrichUsers(ids: string[]): Promise<User[]> {
  // All fetches start immediately; total time ~= slowest single fetch
  return Promise.all(ids.map((id) => fetchUser(id)));
}
// 100 ids at 50ms each = ~50ms
```

## When Sequential Awaiting Inside a Loop Is Correct

```typescript
// Correct: each step genuinely depends on the previous result
async function replayEvents(events: Event[]): Promise<State> {
  let state = initialState();
  for (const event of events) {
    state = await applyEvent(state, event); // must be in order
  }
  return state;
}

// Correct: intentionally throttling requests to respect a rate limit
async function pollUntilRateLimited(urls: string[]) {
  for (const url of urls) {
    await fetch(url);
    await sleep(RATE_LIMIT_INTERVAL_MS);
  }
}
```

The distinguishing question is: does iteration *i+1* need the result of iteration *i*? If not, batch with `Promise.all` (or a concurrency limiter for very large lists) instead of awaiting inside the loop.

## See Also

- [async-promise-all-parallel](async-promise-all-parallel.md) - Use `Promise.all` to run independent async work concurrently
- [async-concurrency-limit](async-concurrency-limit.md) - Bound concurrency with a limiter when processing large batches
- [async-avoid-async-foreach](async-avoid-async-foreach.md) - Avoid `Array.prototype.forEach` with an async callback
