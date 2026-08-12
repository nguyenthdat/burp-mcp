# node-stream-over-buffer

> Use streams for large data — avoid loading everything into memory at once

## Why It Matters

Loading large files or HTTP responses entirely into memory with `readFile()` or `response.json()` can exhaust memory and crash the process. Streams process data incrementally, keeping memory usage constant regardless of data size. For files > 100MB or unbounded data sources, streams are the only safe approach.

## Bad

```js
// Loads entire file into memory — OOM with large files
import { readFile } from 'node:fs/promises';

async function processFile(path) {
  const content = await readFile(path, 'utf8');  // Entire file in memory
  const lines = content.split('\n');
  for (const line of lines) {
    await processLine(line);
  }
}

// Loads entire response body
const users = await fetch('/api/users').then(r => r.json());
// If there are 10 million users, server crashes
```

## Good

```js
// Stream processing — constant memory
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';

async function processFile(path) {
  const stream = createReadStream(path, 'utf8');
  const rl = createInterface({ input: stream });

  for await (const line of rl) {
    await processLine(line);
  }
}

// Stream large HTTP responses
import { get } from 'node:https';

function streamLargeFile(url, outputPath) {
  return new Promise((resolve, reject) => {
    get(url, (response) => {
      const writer = createWriteStream(outputPath);
      response.pipe(writer);
      writer.on('finish', resolve);
      writer.on('error', reject);
    });
  });
}
```

## Node.js Stream Pipeline

```js
import { pipeline } from 'node:stream/promises';
import { createReadStream, createWriteStream } from 'node:fs';
import { Transform } from 'node:stream';

const uppercase = new Transform({
  transform(chunk, encoding, callback) {
    callback(null, chunk.toString().toUpperCase());
  },
});

await pipeline(
  createReadStream('input.txt'),
  uppercase,
  createWriteStream('output.txt'),
);

console.log('Pipeline complete');
```

## When Exceptions Apply

For files under 10MB or when you need random access to the entire content, `readFile` is simpler and fine. Use streams when data size is unbounded or exceeds available memory.

## See Also

- [perf-json-parse-stream](./perf-json-parse-stream.md) - Streaming JSON parsing
- [sec-input-size-limits](./sec-input-size-limits.md) - Limit input sizes
