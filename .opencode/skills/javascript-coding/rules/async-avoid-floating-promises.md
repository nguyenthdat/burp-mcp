# async-avoid-floating-promises

> Never create a promise without handling its resolution or rejection

## Why It Matters

A "floating promise" is created but never awaited, returned, or caught. Its errors are silently swallowed (or crash the process in Node.js 15+). Floating promises are a common source of heisenbugs — errors that appear intermittently but leave no stack trace. Every promise must have a clear owner responsible for its completion.

## Bad

```js
// Floating — no one handles this promise
async function handleRequest(req, res) {
  logToAnalytics(req);  // Returns a promise, never awaited or caught

  const data = await processRequest(req);
  res.json(data);
}

// Floating in constructor
class Service {
  constructor() {
    this.initialize();  // async — errors are lost
  }
}

// Floating in event handler
emitter.on('data', (chunk) => {
  saveChunk(chunk);  // Promise lost
});
```

## Good

```js
// Await in async context
async function handleRequest(req, res) {
  await logToAnalytics(req);
  const data = await processRequest(req);
  res.json(data);
}

// Fire-and-forget with explicit .catch()
async function handleRequest(req, res) {
  logToAnalytics(req).catch(err => console.error('Analytics failed:', err));
  const data = await processRequest(req);
  res.json(data);
}

// Use async init pattern instead of async constructor
class Service {
  static async create() {
    const service = new Service();
    await service.initialize();
    return service;
  }
}

// Use void to signal intentional fire-and-forget (TypeScript-friendly)
emitter.on('data', (chunk) => {
  void saveChunk(chunk).catch(err => console.error('Save failed:', err));
});
```

## When Exceptions Apply

The only acceptable floating promise is one that is intentionally fire-and-forget with an explicit `.catch()` handler that suppresses or logs the error.

## See Also

- [async-error-swallowed](./async-error-swallowed.md) - Handle promise rejections
- [async-return-promise](./async-return-promise.md) - Always return promises from async functions
