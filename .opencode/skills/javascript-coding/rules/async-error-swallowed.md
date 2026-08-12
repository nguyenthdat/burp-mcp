# async-error-swallowed

> Always handle promise rejections — never leave a promise without .catch()

## Why It Matters

An unhandled promise rejection causes the Node.js process to emit an `unhandledRejection` event and, starting in Node.js 15, terminates the process. Silent promise rejections are one of the hardest bugs to debug because they leave no trace and produce no visible error.

## Bad

```js
// Rejection is silently swallowed
async function process() {
  doSomethingAsync();  // No await, no .catch()
}

// In event handlers
button.onclick = () => {
  fetchData().then(updateUI);  // No .catch() — rejection is lost
};

// In Promise chains
fetch(url)
  .then(res => res.json())
  .then(data => process(data));  // No .catch() at end of chain
```

## Good

```js
// Await and handle
async function process() {
  try {
    await doSomethingAsync();
  } catch (err) {
    console.error('Processing failed:', err);
  }
}

// Always add .catch()
button.onclick = () => {
  fetchData()
    .then(updateUI)
    .catch(err => console.error('Fetch failed:', err));
};

// Top-level catch
fetch(url)
  .then(res => res.json())
  .then(data => process(data))
  .catch(err => console.error('Pipeline failed:', err));
```

## When Exceptions Apply

In fire-and-forget scenarios (logging, analytics), use `.catch()` with a no-op handler or intentional suppression:

```js
analytics.track(event).catch(() => {});  // Explicitly ignore — we don't care
```

## See Also

- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Never create unhandled promises
- [err-global-handlers](./err-global-handlers.md) - Set global unhandledRejection handler
