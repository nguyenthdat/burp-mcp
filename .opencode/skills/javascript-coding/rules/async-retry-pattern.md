# async-retry-pattern

> Implement exponential backoff for transient failures

## Why It Matters

Network calls, database queries, and external API requests can fail transiently due to rate limiting, network blips, or temporary server overload. Retrying with exponential backoff and jitter allows your application to recover gracefully without overwhelming the downstream service or causing thundering herd problems.

## Bad

```js
// No retry — one transient failure kills the operation
async function fetchUser(id) {
  const response = await fetch(`/api/users/${id}`);
  return await response.json();
}

// Immediate retry without backoff — can amplify load
async function fetchUser(id) {
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      const response = await fetch(`/api/users/${id}`);
      return await response.json();
    } catch (err) {
      // No delay — immediate retry may overwhelm the server
    }
  }
}
```

## Good

```js
async function fetchWithRetry(url, { maxRetries = 3, baseDelay = 1000 } = {}) {
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(url);
      if (!response.ok && response.status >= 500) {
        throw new Error(`Server error: ${response.status}`);
      }
      return await response.json();
    } catch (err) {
      if (attempt === maxRetries) throw err;

      const delay = baseDelay * 2 ** attempt + Math.random() * 1000;
      console.warn(`Attempt ${attempt + 1} failed, retrying in ${Math.round(delay)}ms`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}
```

## Selective Retry

Only retry on transient errors — not on 4xx client errors:

```js
function isRetryable(err) {
  if (err.name === 'AbortError') return true;
  if (err.cause?.code === 'ECONNRESET') return true;
  if (err.cause?.code === 'ETIMEDOUT') return true;
  return false;
}

async function fetchWithRetry(url, opts = {}) {
  for (let i = 0; i <= opts.maxRetries; i++) {
    try {
      return await fetch(url, { signal: opts.signal });
    } catch (err) {
      if (i === opts.maxRetries || !isRetryable(err)) throw err;
      await delay(2 ** i * 1000);
    }
  }
}
```

## When Exceptions Apply

Don't retry operations that are not idempotent (e.g., payment processing, order creation) unless your API provides idempotency keys.

## See Also

- [async-timeout-pattern](./async-timeout-pattern.md) - Combine timeout with retry
- [async-promise-allsettled](./async-promise-allsettled.md) - Handle batch failures
