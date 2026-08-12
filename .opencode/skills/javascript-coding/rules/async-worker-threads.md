# async-worker-threads

> Use worker_threads for CPU-intensive work to avoid blocking the event loop

## Why It Matters

Node.js runs JavaScript on a single thread. CPU-intensive operations (image processing, cryptography, heavy computation, parsing large data) block the event loop, causing unresponsiveness. Worker threads run JavaScript in parallel on separate threads, keeping the main thread free for I/O.

## Bad

```js
// Blocks the event loop during computation
function processImage(buffer) {
  // Heavy image processing — 500ms
  const result = sharp(buffer).resize(1920, 1080).toBuffer();
  return result;
}

// In a request handler — blocks all other requests
app.post('/upload', (req, res) => {
  const processed = processImage(req.body);  // 500ms block
  res.send(processed);
});
```

## Good

```js
// Offload to a worker thread
import { Worker } from 'node:worker_threads';

function processImage(buffer) {
  return new Promise((resolve, reject) => {
    const worker = new Worker('./image-worker.mjs', {
      workerData: buffer,
    });
    worker.on('message', resolve);
    worker.on('error', reject);
    worker.on('exit', (code) => {
      if (code !== 0) reject(new Error(`Worker stopped with exit code ${code}`));
    });
  });
}

// image-worker.mjs
import { parentPort, workerData } from 'node:worker_threads';
import sharp from 'sharp';

const result = await sharp(workerData).resize(1920, 1080).toBuffer();
parentPort.postMessage(result);
```

## Worker Pool Pattern

Creating workers per request is expensive. Use a pool:

```js
import { StaticPool } from 'node:worker_threads';

const pool = new StaticPool({
  size: 4,
  task: './heavy-task.mjs',
});

const result = await pool.exec({ data: input });
```

## When Exceptions Apply

For small computations (< 10ms), the overhead of worker thread communication outweighs the benefit. Use worker threads when processing takes > 50ms.

## See Also

- [async-no-blocking-loop](./async-no-blocking-loop.md) - Don't block the event loop
- [perf-lazy-load](./perf-lazy-load.md) - Lazy-load heavy modules
