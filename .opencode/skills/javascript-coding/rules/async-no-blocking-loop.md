# async-no-blocking-loop

> Don't use synchronous fs/math/crypto in async contexts

## Why It Matters

Synchronous I/O and CPU-intensive operations block the Node.js event loop, preventing any other requests from being handled. In a server context, this causes latency spikes and degraded throughput. Always use async versions of fs, crypto, and other potentially blocking APIs inside async functions.

## Bad

```js
import { readFileSync, writeFileSync } from 'node:fs';

async function processFile(path) {
  const data = readFileSync(path, 'utf8');   // Blocks the event loop
  const result = expensiveCrypto(data);       // Blocks
  writeFileSync(path, result);                // Blocks
  return result;
}
```

## Good

```js
import { readFile, writeFile } from 'node:fs/promises';

async function processFile(path) {
  const data = await readFile(path, 'utf8');
  const result = await expensiveCrypto(data);
  await writeFile(path, result);
  return result;
}

// For CPU-bound work, offload to a worker thread
import { Worker } from 'node:worker_threads';

function expensiveCrypto(data) {
  return new Promise((resolve, reject) => {
    const worker = new Worker('./crypto-worker.mjs', { workerData: data });
    worker.on('message', resolve);
    worker.on('error', reject);
  });
}
```

## When Exceptions Apply

Synchronous methods are acceptable in CLI scripts, build tools, or startup code that runs once:

```js
// Acceptable at startup
const config = JSON.parse(readFileSync('./config.json', 'utf8'));
```

## See Also

- [async-worker-threads](./async-worker-threads.md) - Use worker_threads for CPU-intensive work
- [node-fs-async-over-sync](./node-fs-async-over-sync.md) - Prefer fs/promises over fs sync
