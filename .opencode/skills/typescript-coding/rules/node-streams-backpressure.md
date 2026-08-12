# node-streams-backpressure

> Use streams with backpressure for large I/O instead of buffering in memory

## Why It Matters

Reading an entire file, HTTP response body, or database export into memory (`fs.readFileSync`, `await response.text()`, array-of-all-rows) means memory usage scales with input size — a 2 GB file needs roughly 2 GB of heap, and concurrent requests multiply that. Node's streams process data in chunks and propagate backpressure automatically: if the destination (a slow disk, a slow network socket) can't keep up, `.pipe()` pauses the source instead of piling up unbounded chunks in memory. Ignoring backpressure (e.g., writing to a stream without checking the return value, or manually reading a whole stream into a buffer) is a common cause of OOM crashes under load, and it's avoidable — streaming is a first-class part of the built-in API, not an advanced technique.

## Bad

```typescript
import fs from 'node:fs';

// Reads the entire file into memory before writing any of it back out
async function copyLargeFile(src: string, dest: string) {
  const data = await fs.promises.readFile(src); // whole file in RAM
  await fs.promises.writeFile(dest, data);
}

// Buffers an entire HTTP response body before processing
async function downloadAndProcess(url: string) {
  const res = await fetch(url);
  const buffer = await res.arrayBuffer(); // could be gigabytes
  process(buffer);
}
```

## Good

```typescript
import fs from 'node:fs';
import { pipeline } from 'node:stream/promises';
import { Transform } from 'node:stream';

// pipeline() streams chunk-by-chunk and respects backpressure automatically
async function copyLargeFile(src: string, dest: string) {
  await pipeline(fs.createReadStream(src), fs.createWriteStream(dest));
}

// Process an HTTP response as a stream instead of buffering it whole
async function downloadAndProcess(url: string) {
  const res = await fetch(url);
  if (!res.body) return;

  const upperCase = new Transform({
    transform(chunk, _enc, callback) {
      callback(null, chunk.toString().toUpperCase());
    },
  });

  await pipeline(res.body as unknown as NodeJS.ReadableStream, upperCase, fs.createWriteStream('out.txt'));
}
```

## Why `pipeline()` Over Manual `.pipe()`

`stream.pipeline()` (from `node:stream/promises`) handles error propagation and cleanup for you — if any stream in the chain errors or is destroyed, all the others are destroyed too. Manual `.pipe()` chains leave you responsible for listening to `'error'` on every stream and manually calling `.destroy()`, which is easy to get wrong and leaks file descriptors when it is.

```typescript
// Manual .pipe() — error-prone, easy to leak file descriptors on error
src.pipe(dest); // an error on 'src' does not destroy 'dest'
```

## See Also

- [node-avoid-sync-fs](node-avoid-sync-fs.md) - Avoid synchronous `fs` calls on a server's request path
- [perf-avoid-json-parse-large](perf-avoid-json-parse-large.md) - Avoid parsing very large JSON payloads synchronously
- [async-generator-streams](async-generator-streams.md) - Use async generators to model streaming data
