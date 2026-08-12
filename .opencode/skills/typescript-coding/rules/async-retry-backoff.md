# async-retry-backoff

> Retry transient failures with exponential backoff and jitter

## Why It Matters

Network calls, database connections, and third-party APIs fail transiently — a dropped connection, a momentary 503, a rate limit. Retrying immediately in a tight loop wastes resources and, at scale, can create a "thundering herd" that makes an already-struggling downstream service worse. Exponential backoff spreads retries out over increasing intervals, and adding jitter (randomness) prevents many clients from retrying in lockstep and re-overwhelming the service at the same instant.

## Bad

```typescript
async function fetchWithRetry(url: string): Promise<Response> {
  for (let i = 0; i < 5; i++) {
    try {
      return await fetch(url);
    } catch {
      // Retries instantly, 5 times in a row, no delay at all
    }
  }
  throw new Error("failed after 5 attempts");
}
```

## Good

```typescript
interface RetryOptions {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
}

async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  { maxAttempts, baseDelayMs, maxDelayMs }: RetryOptions,
): Promise<T> {
  let lastError: unknown;

  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
      if (attempt === maxAttempts - 1) break;

      const exponential = Math.min(baseDelayMs * 2 ** attempt, maxDelayMs);
      const jitter = Math.random() * exponential * 0.5;
      await new Promise((resolve) => setTimeout(resolve, exponential + jitter));
    }
  }

  throw lastError;
}

async function fetchWithRetry(url: string): Promise<Response> {
  return retryWithBackoff(() => fetch(url), {
    maxAttempts: 5,
    baseDelayMs: 200,
    maxDelayMs: 5000,
  });
}
```

## Only Retry What's Actually Retryable

```typescript
function isRetryable(err: unknown): boolean {
  if (err instanceof HttpError) {
    // 429 (rate limited) and 5xx (server error) are worth retrying;
    // 4xx client errors like 400/401/404 will fail again identically.
    return err.status === 429 || err.status >= 500;
  }
  return err instanceof NetworkError; // e.g. connection reset, DNS failure
}
```

Blindly retrying every error — including validation failures or auth errors — wastes attempts on failures that will never succeed and can mask real bugs.

## See Also

- [async-timeout-race](async-timeout-race.md) - Implement timeouts by racing a promise against a timer
- [err-custom-error-class](err-custom-error-class.md) - Define typed errors so retry logic can distinguish failure kinds
- [async-abort-controller](async-abort-controller.md) - Use `AbortController` to make async operations cancellable
