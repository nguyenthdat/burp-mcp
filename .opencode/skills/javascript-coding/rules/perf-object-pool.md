# perf-object-pool

> Use object pooling for frequently created and discarded objects to reduce GC pressure

## Why It Matters

Creating and garbage-collecting thousands of short-lived objects per second causes GC pauses that block the event loop. Object pooling pre-allocates a set of reusable objects, eliminating allocation and GC overhead. This pattern is critical for high-throughput servers, game engines, and real-time data processing.

## Bad

```js
// New object per request — GC churn
function processRequest(data) {
  const buffer = Buffer.alloc(4096);  // Allocation per request
  const parser = new JSONParser();     // Allocation per request
  const result = { status: 'ok', data: null };  // Allocation per request

  // ... process

  return result;
}
// After 1M requests: millions of allocations and GC pauses
```

## Good

```js
class BufferPool {
  #pool = [];
  #size;

  constructor(size, initialCount = 10) {
    this.#size = size;
    for (let i = 0; i < initialCount; i++) {
      this.#pool.push(Buffer.allocUnsafe(size));
    }
  }

  acquire() {
    return this.#pool.pop() || Buffer.allocUnsafe(this.#size);
  }

  release(buf) {
    if (buf.length === this.#size) {
      this.#pool.push(buf);
    }
  }
}

const pool = new BufferPool(4096, 50);

function processRequest(data) {
  const buffer = pool.acquire();
  try {
    // ... process using buffer
    return { status: 'ok', data: result };
  } finally {
    pool.release(buffer);
  }
}
```

## When Exceptions Apply

Object pooling adds complexity. Only apply it when profiling shows GC is a bottleneck. For most applications, the built-in garbage collector handles short-lived objects efficiently.

## See Also

- [perf-memoize](./perf-memoize.md) - Memoize expensive functions
- [node-http-agent](./node-http-agent.md) - Connection pooling
