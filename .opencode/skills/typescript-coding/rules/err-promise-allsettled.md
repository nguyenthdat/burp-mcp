# err-promise-allsettled

> Use `Promise.allSettled` when independent operations may fail without aborting others

## Why It Matters

`Promise.all` rejects as soon as any one promise rejects, immediately discarding the results of every other promise in the batch — even ones that succeeded or were about to. When the operations are independent (notifying five webhooks, fetching data from three unrelated services), that behavior throws away useful partial results and makes one flaky call take down the whole batch. `Promise.allSettled` waits for every promise to settle and reports each outcome individually, so you can act on successes and handle failures separately.

## Bad

```typescript
async function notifyAllWebhooks(urls: string[], payload: unknown) {
  // If any single webhook fails, Promise.all rejects immediately —
  // we never find out whether the other 9 succeeded or not
  await Promise.all(urls.map((url) => sendWebhook(url, payload)));
}
```

## Good

```typescript
async function notifyAllWebhooks(urls: string[], payload: unknown) {
  const results = await Promise.allSettled(
    urls.map((url) => sendWebhook(url, payload)),
  );

  const failures = results
    .map((result, i) => ({ result, url: urls[i] }))
    .filter((r): r is { result: PromiseRejectedResult; url: string } =>
      r.result.status === "rejected",
    );

  if (failures.length > 0) {
    logger.warn("some webhooks failed", {
      failed: failures.map((f) => ({ url: f.url, reason: f.result.reason })),
    });
  }

  return {
    succeeded: results.filter((r) => r.status === "fulfilled").length,
    failed: failures.length,
  };
}
```

## Result Shape

```typescript
type SettledResult<T> =
  | { status: "fulfilled"; value: T }
  | { status: "rejected"; reason: unknown };

// Promise.allSettled<T>(promises: Promise<T>[]): Promise<SettledResult<T>[]>
```

## Choosing Between `all`, `allSettled`, `any`, and `race`

| API | Resolves when | Use case |
|---|---|---|
| `Promise.all` | All succeed; rejects on first failure | Operations are dependent — one failure invalidates the batch |
| `Promise.allSettled` | All settle (success or failure) | Independent operations; partial success is meaningful |
| `Promise.any` | First one succeeds; rejects only if all fail | Redundant sources (e.g. multiple CDN mirrors), any success is enough |
| `Promise.race` | First one settles (success or failure) | Timeouts, cancellation races |

## See Also

- [async-promise-all-parallel](async-promise-all-parallel.md) - Run independent async operations in parallel with Promise.all
- [async-timeout-race](async-timeout-race.md) - Use Promise.race to enforce a timeout on an async operation
- [err-specific-catch](err-specific-catch.md) - Catch and handle specific error types instead of a blanket catch-all
