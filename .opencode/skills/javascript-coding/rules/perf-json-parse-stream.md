# perf-json-parse-stream

> Use a streaming JSON parser (JSONStream, clarinet, stream-json) for large JSON payloads

## Why It Matters

`JSON.parse()` loads the entire JSON string into memory and builds the complete object tree. For multi-MB or GB JSON files (logs, exports, data dumps), this can exhaust memory and crash the process. Streaming parsers emit events as they parse, keeping memory usage constant regardless of file size.

## Bad

```js
// Entire file loaded into memory
import { readFile } from 'node:fs/promises';

async function processLargeJSON(path) {
  const raw = await readFile(path, 'utf8');
  const data = JSON.parse(raw);  // OOM if file is 2GB
  for (const record of data.records) {
    await processRecord(record);
  }
}
```

## Good

```js
// Stream JSON parsing — constant memory
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';

// For newline-delimited JSON (NDJSON)
async function processNDJSON(path) {
  const stream = createReadStream(path);
  const rl = createInterface({ input: stream });

  for await (const line of rl) {
    if (line.trim()) {
      const record = JSON.parse(line);  // One record at a time
      await processRecord(record);
    }
  }
}

// For large JSON arrays: use a streaming parser
import { parser } from 'stream-json';
import { streamArray } from 'stream-json/streamers/StreamArray';

async function processJSONArray(path) {
  const pipeline = createReadStream(path)
    .pipe(parser())
    .pipe(streamArray());

  for await (const { value } of pipeline) {
    await processRecord(value);  // One array element at a time
  }
}
```

## When Exceptions Apply

For JSON payloads under 10MB, `JSON.parse()` is simpler and faster. Use streaming for files larger than available memory or for unbounded API response streams.

## See Also

- [node-stream-over-buffer](./node-stream-over-buffer.md) - Streams for large data
- [sec-input-size-limits](./sec-input-size-limits.md) - Limit input sizes
