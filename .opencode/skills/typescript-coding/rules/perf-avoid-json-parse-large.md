# perf-avoid-json-parse-large

> Avoid blocking synchronous JSON parsing of large payloads on hot paths

## Why It Matters

`JSON.parse` and `JSON.stringify` are synchronous and single-threaded; parsing a multi-megabyte payload can block the Node.js event loop or the browser main thread for tens to hundreds of milliseconds, during which no other request can be handled and no UI can update. On a server, this directly caps throughput under load since one large request stalls every other concurrent connection; in a browser, it causes visible input lag and dropped frames.

## Bad

```typescript
// A single large upload blocks the event loop for the duration of the parse,
// stalling every other concurrent request on this Node.js process.
app.post("/import", async (req, res) => {
  const body = await readBody(req); // e.g. a 50MB JSON export
  const data = JSON.parse(body); // blocks synchronously
  await processImport(data);
  res.sendStatus(200);
});
```

## Good

```typescript
import { parser } from "stream-json";
import { streamArray } from "stream-json/streamers/StreamArray";

// Stream-parse large JSON incrementally instead of buffering
// the whole payload and blocking on a single JSON.parse call.
app.post("/import", async (req, res) => {
  const pipeline = req.pipe(parser()).pipe(streamArray());

  for await (const { value } of pipeline) {
    await processImportRecord(value);
  }
  res.sendStatus(200);
});
```

```typescript
// Or, for a large but still in-memory payload, offload the parse
// to a worker thread so it doesn't block the main event loop.
import { Worker } from "node:worker_threads";

function parseInWorker(json: string): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const worker = new Worker("./json-parse-worker.js", { workerData: json });
    worker.once("message", resolve);
    worker.once("error", reject);
  });
}
```

## Guidelines

- For payloads in the low kilobytes, plain `JSON.parse` is fine — this rule targets payloads large enough (typically multi-megabyte) to cause measurable event-loop stalls; profile before assuming a payload is "large."
- Stream-parse with libraries like `stream-json` when the data is naturally array/record-shaped and can be processed incrementally without needing the whole structure in memory at once.
- Offload to a `worker_threads` worker (see `perf-worker-offload`) when the full structure is genuinely needed in memory but the parse itself is the bottleneck.
- On the client side, consider `JSON.parse` inside a Web Worker for the same reason — large API responses parsed on the main thread can cause visible jank in an otherwise smooth UI.

## See Also

- [perf-worker-offload](perf-worker-offload.md) - Offload CPU-heavy work to a worker thread instead of blocking the main thread
- [perf-avoid-blocking-event-loop](perf-avoid-blocking-event-loop.md) - Avoid long synchronous operations that block the event loop
- [node-streams-backpressure](node-streams-backpressure.md) - handle backpressure correctly when streaming large payloads
