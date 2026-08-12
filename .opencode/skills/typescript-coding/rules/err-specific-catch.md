# err-specific-catch

> Catch and handle specific error types instead of a blanket catch-all

## Why It Matters

A single `catch (err) { ... }` block that treats every possible failure the same way ends up either handling all of them wrong, or handling none of them right — a network timeout, a validation failure, and a programming bug (like accessing an undefined property) usually call for completely different responses (retry, show a user-facing message, crash loudly and page someone). Discriminating on error type inside the catch block lets each failure mode get the response it actually needs, and lets unrecognized errors be rethrown rather than silently absorbed into the wrong branch.

## Bad

```typescript
async function loadProfile(userId: string) {
  try {
    return await api.getUser(userId);
  } catch (err) {
    // Every possible failure gets the same generic treatment
    console.log("something went wrong");
    return null;
  }
}
```

## Good

```typescript
class NotFoundError extends Error {}
class RateLimitError extends Error {
  constructor(public readonly retryAfterMs: number) {
    super("rate limited");
  }
}

async function loadProfile(userId: string): Promise<UserProfile | null> {
  try {
    return await api.getUser(userId);
  } catch (err) {
    if (err instanceof NotFoundError) {
      return null; // expected, not an error worth logging loudly
    }
    if (err instanceof RateLimitError) {
      await sleep(err.retryAfterMs);
      return loadProfile(userId); // retry with the server-provided backoff
    }
    if (err instanceof TypeError) {
      // Likely a bug in our own code (e.g. bad destructuring) — don't hide it
      throw err;
    }
    logger.error("unexpected error loading profile", { userId, error: err });
    throw err;
  }
}
```

## Discriminating Non-Error Rejections

Not everything thrown is an `Error` instance (fetch aborts, third-party libraries that reject with plain objects), so check shape defensively:

```typescript
function isAbortError(err: unknown): err is DOMException {
  return err instanceof DOMException && err.name === "AbortError";
}

try {
  await fetch(url, { signal: controller.signal });
} catch (err) {
  if (isAbortError(err)) {
    return; // expected — request was intentionally cancelled
  }
  throw err;
}
```

## Decision Table

| Error type | Typical handling |
|---|---|
| Expected domain error (`NotFoundError`, `ValidationError`) | Handle locally, return a fallback or user-facing message |
| Transient/retryable (`RateLimitError`, network timeout) | Retry with backoff |
| Programmer error (`TypeError`, assertion failure) | Rethrow / let it crash — fixing the bug is the real solution |
| Unknown/unclassified | Log with full context, then rethrow |

## See Also

- [err-custom-error-class](err-custom-error-class.md) - Extend Error with custom subclasses that carry structured context
- [err-never-swallow](err-never-swallow.md) - Never silently swallow errors in empty catch blocks
- [err-typed-catch-unknown](err-typed-catch-unknown.md) - Type the catch binding as unknown and narrow before use
