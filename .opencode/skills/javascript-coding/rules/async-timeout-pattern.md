# async-timeout-pattern

> Wrap async operations with a timeout to prevent hanging indefinitely

## Why It Matters

Network requests, database queries, and other async operations can hang indefinitely due to network issues, deadlocks, or resource exhaustion. Wrapping operations with a timeout ensures your application fails fast and can recover, rather than accumulating stuck requests.

## Bad

```js
// May hang forever if the server never responds
async function fetchData(url) {
  const response = await fetch(url);
  return await response.json();
}
```

## Good

```js
function withTimeout(promise, ms, message = 'Operation timed out') {
  return Promise.race([
    promise,
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error(message)), ms)
    ),
  ]);
}

async function fetchData(url, timeoutMs = 5000) {
  const response = await withTimeout(fetch(url), timeoutMs, 'Fetch timed out');
  return await response.json();
}
```

## Using AbortSignal (Preferred)

```js
async function fetchWithTimeout(url, ms) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), ms);

  try {
    const response = await fetch(url, { signal: controller.signal });
    return await response.json();
  } finally {
    clearTimeout(timer);
  }
}
```

## When Exceptions Apply

Don't add timeouts to operations where the duration is inherently unpredictable — e.g., large file uploads, batch processing jobs, or database migrations. Instead, use progress reporting and configurable timeout limits.

## See Also

- [async-abort-control](./async-abort-control.md) - Use AbortController for cancellation
- [async-retry-pattern](./async-retry-pattern.md) - Retry after transient failures
