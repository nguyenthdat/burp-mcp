# perf-avoid-blocking-event-loop

> Avoid long synchronous operations that block the event loop

## Why It Matters

Node.js (and the browser's main thread) processes work on a single thread via an event loop; any synchronous operation that takes a long time to complete — a large `JSON.parse`, a tight computational loop over a huge array, a synchronous filesystem call — blocks every other pending callback, timer, and incoming request until it finishes. On a server, this directly caps throughput and can cause request timeouts for unrelated clients; in a browser, it freezes scrolling, input, and animation.

## Bad

```typescript
// A single request handling a large synchronous computation
// blocks every other concurrent request on this process.
app.get("/report/:year", (req, res) => {
  const rows = loadAllTransactions(req.params.year); // 2 million rows, already in memory
  let total = 0;
  for (const row of rows) {
    total += computeAdjustedAmount(row); // expensive, synchronous, pure CPU work
  }
  res.json({ total });
});

// Synchronous filesystem call blocks the event loop for the duration of the read
const config = fs.readFileSync("./large-config.json", "utf8");
```

## Good

```typescript
// Yield to the event loop periodically during a long synchronous computation
async function computeTotal(rows: Transaction[]): Promise<number> {
  let total = 0;
  const CHUNK_SIZE = 10_000;
  for (let i = 0; i < rows.length; i += CHUNK_SIZE) {
    const chunk = rows.slice(i, i + CHUNK_SIZE);
    for (const row of chunk) {
      total += computeAdjustedAmount(row);
    }
    // Yield back to the event loop so other pending work can run
    await new Promise((resolve) => setImmediate(resolve));
  }
  return total;
}

app.get("/report/:year", async (req, res) => {
  const rows = await loadAllTransactions(req.params.year);
  res.json({ total: await computeTotal(rows) });
});

// Use the async filesystem API instead of the *Sync variant
const config = await fs.promises.readFile("./large-config.json", "utf8");
```

## Guidelines

- Always prefer the async variant of Node's built-in APIs (`fs.promises.readFile` over `fs.readFileSync`) on any code path that serves concurrent requests (see `node-avoid-sync-fs`).
- For genuinely CPU-bound work that can't be chunked or made async, move it to a worker thread (see `perf-worker-offload`) rather than trying to interleave it with `setImmediate`.
- Yielding with `setImmediate`/`await Promise.resolve()` inside a chunked loop keeps a single computation from starving the event loop, but it doesn't make the total work any faster — it only lets other requests interleave.
- Monitor event loop lag in production (e.g. with the `perf_hooks` `monitorEventLoopDelay` API or an APM tool) to catch blocking operations that weren't obvious in code review.

## See Also

- [perf-worker-offload](perf-worker-offload.md) - Offload CPU-heavy work to a worker thread instead of blocking the main thread
- [node-avoid-sync-fs](node-avoid-sync-fs.md) - avoid synchronous filesystem calls on request-handling paths
- [perf-avoid-json-parse-large](perf-avoid-json-parse-large.md) - Avoid blocking synchronous JSON parsing of large payloads on hot paths
