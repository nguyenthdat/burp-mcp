# async-abort-control

> Use AbortController for cancellable async operations

## Why It Matters

Without cancellation, long-running async operations continue consuming resources even after the caller no longer needs the result. `AbortController` and `AbortSignal` provide a standard mechanism to cancel fetch requests, timers, streams, and custom async work. This prevents memory leaks and wasted compute.

## Bad

```js
// Uncancelable — request continues even if user navigates away
async function loadDashboard() {
  const data = await fetch('/api/dashboard');
  return await data.json();
}

// Timer keeps running
const id = setTimeout(() => cleanup(), 60_000);
// No way to cancel from outside
```

## Good

```js
// Cancelable fetch
async function loadDashboard(signal) {
  const response = await fetch('/api/dashboard', { signal });
  return await response.json();
}

const controller = new AbortController();

// Cancel after timeout or user action
setTimeout(() => controller.abort(), 10_000);
loadDashboard(controller.signal).catch(err => {
  if (err.name === 'AbortError') {
    console.log('Request cancelled');
  }
});

// Cancelable custom async work
function delay(ms, signal) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(resolve, ms);
    signal?.addEventListener('abort', () => {
      clearTimeout(timer);
      reject(new DOMException('Aborted', 'AbortError'));
    }, { once: true });
  });
}
```

## When Exceptions Apply

Cancellation isn't needed for very short-lived operations (< 100ms) or for operations that must complete (e.g., critical database writes, payment processing).

## See Also

- [async-timeout-pattern](./async-timeout-pattern.md) - Wrap async operations with timeout
- [node-signal-handling](./node-signal-handling.md) - Handle process signals for graceful shutdown
