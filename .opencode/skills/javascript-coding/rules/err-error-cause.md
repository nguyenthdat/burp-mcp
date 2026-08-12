# err-error-cause

> Use `{ cause: err }` (ES2022) for error chaining

## Why It Matters

When catching and rethrowing errors, you lose the original error context unless you explicitly chain it. The `cause` option in the Error constructor preserves the originating error, giving callers access to the full causal chain. This is essential for debugging and for logging systems that need root-cause analysis.

## Bad

```js
// Original error is lost — no way to trace the root cause
async function loadConfig(path) {
  try {
    return JSON.parse(await readFile(path, 'utf8'));
  } catch (err) {
    throw new Error(`Failed to load config from ${path}`);
  }
}

// Manual chaining with a separate property
try {
  await connect();
} catch (err) {
  const wrapped = new Error('Connection failed');
  wrapped.original = err;  // Non-standard, easy to miss
  throw wrapped;
}
```

## Good

```js
// ES2022 — standard error chaining
async function loadConfig(path) {
  try {
    return JSON.parse(await readFile(path, 'utf8'));
  } catch (err) {
    throw new Error(`Failed to load config from ${path}`, { cause: err });
  }
}

// Caller can inspect the full chain
try {
  const config = await loadConfig('./config.json');
} catch (err) {
  console.error(err.message);           // "Failed to load config from ./config.json"
  console.error(err.cause?.message);    // "Unexpected token } in JSON at position 42"
  console.error(err.cause?.cause);      // Further down the chain if nested
}
```

## Chaining Through Async Operations

```js
async function fetchUserData(userId) {
  try {
    const response = await fetch(`/api/users/${userId}`);
    return await response.json();
  } catch (err) {
    throw new Error(`Failed to fetch user ${userId}`, { cause: err });
  }
}

async function getDashboard(userId) {
  try {
    const [profile, settings] = await Promise.all([
      fetchUserData(userId),
      fetchSettings(userId),
    ]);
    return { profile, settings };
  } catch (err) {
    throw new Error(`Failed to build dashboard for ${userId}`, { cause: err });
  }
}
```

## When Exceptions Apply

Don't chain errors when the original error contains sensitive information that should not leak to the caller. Sanitize the cause before rethrowing.

## See Also

- [err-custom-error-classes](./err-custom-error-classes.md) - Custom error types
- [err-no-throw-string](./err-no-throw-string.md) - Always throw Error instances
- [err-structured-error-handling](./err-structured-error-handling.md) - Error hierarchy
