# lint-no-floating-promises-rule

> Enable `@typescript-eslint/no-floating-promises`

## Why It Matters

A Promise that is created but never `await`ed, `.then()`-chained, or explicitly discarded is a "floating" promise: if it rejects, the rejection becomes an unhandled promise rejection, which in Node.js by default crashes the process (`--unhandled-rejections=strict` is the current default) and in a browser silently vanishes into the console. Because forgetting an `await` compiles without any error — an `async` function call without `await` is syntactically valid and just returns an ignored `Promise<T>` — this class of bug is invisible without a linter watching for it specifically. This rule requires every promise-returning expression to be awaited, handled, or explicitly voided, closing off the most common source of silently-swallowed async errors.

## Bad

```typescript
async function saveUser(user: User) {
  db.save(user); // forgot `await` — a rejection here is unhandled and unlogged
  return { ok: true };
}

function handleClick() {
  fetchData(); // async function called without await in a sync context
}
```

## Good

```javascript
// eslint.config.js
export default tseslint.config({
  rules: {
    '@typescript-eslint/no-floating-promises': 'error',
  },
});
```

```typescript
async function saveUser(user: User) {
  await db.save(user); // rejection now propagates to the caller
  return { ok: true };
}

function handleClick() {
  void fetchData().catch((err) => logger.error({ err }, 'fetch failed'));
  // `void` explicitly marks this as an intentionally-unawaited fire-and-forget
}
```

## Configuring Intentional Fire-and-Forget

```jsonc
// If you deliberately fire-and-forget in specific spots (e.g. telemetry),
// allow the `void` operator to satisfy the rule instead of disabling it:
{
  "rules": {
    "@typescript-eslint/no-floating-promises": [
      "error",
      { "ignoreVoid": true }
    ]
  }
}
```

```typescript
// Telemetry failures shouldn't block the main flow, but must still be logged
void trackEvent('checkout_completed').catch((err) => logger.warn({ err }, 'telemetry failed'));
```

## See Also

- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise unawaited without an explicit handler
- [async-void-operator](async-void-operator.md) - Use `void` to explicitly mark an intentionally unawaited promise
- [err-unhandled-rejection](err-unhandled-rejection.md) - Handle unhandled promise rejections at the process level
