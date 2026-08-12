# async-avoid-for-await-misuse

> Don't use for-await-of on synchronous or non-async iterables

## Why It Matters

`for await...of` is designed for async iterables — objects implementing `Symbol.asyncIterator`. Using it on a regular array or sync iterable triggers unnecessary microtask scheduling, adding latency. Use `for...of` for synchronous iterables and reserve `for await...of` for streams, async generators, and ReadableStreams.

## Bad

```js
// Unnecessary async iteration — adds microtask overhead
async function processItems(items) {
  for await (const item of items) {  // items is a plain array
    await save(item);
  }
}

// Wrapped array in an async iterable for no reason
async function* makeAsync(arr) {
  for (const item of arr) yield item;
}
for await (const x of makeAsync(data)) { /* ... */ }
```

## Good

```js
// Use for...of for sync iterables
async function processItems(items) {
  for (const item of items) {
    await save(item);
  }
}

// for await...of is correct for true async iterables
async function* fetchPages(baseUrl) {
  let page = 1;
  while (true) {
    const data = await fetch(`${baseUrl}?page=${page}`).then(r => r.json());
    if (data.length === 0) break;
    yield data;
    page++;
  }
}

for await (const page of fetchPages('/api/items')) {
  await processPage(page);
}

// Also correct for Node.js Readable streams
import { createReadStream } from 'node:fs';

const stream = createReadStream('data.csv');
for await (const chunk of stream) {
  await processChunk(chunk);
}
```

## When Exceptions Apply

`for await...of` works on sync iterables, so linters may not flag it. It's not a bug — just a performance issue. When processing massive arrays in hot paths, the microtask overhead can be significant.

## See Also

- [async-parallel-over-sequential](./async-parallel-over-sequential.md) - Concurrent processing
- [node-stream-over-buffer](./node-stream-over-buffer.md) - Stream large data
