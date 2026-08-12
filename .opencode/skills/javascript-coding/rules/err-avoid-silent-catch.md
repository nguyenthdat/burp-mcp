# err-avoid-silent-catch

> Always log or rethrow caught errors — never silently swallow them

## Why It Matters

An empty `catch` block is one of the most dangerous anti-patterns. It hides failures, making debugging impossible. Production incidents often trace back to a silently swallowed error that masked a cascading failure. Even if you choose not to propagate an error, at minimum log it.

## Bad

```js
// Silent — error disappears without a trace
try {
  await saveToDatabase(record);
} catch (err) {
  // Empty block — error is gone forever
}

// Logging without useful context
try {
  await process(order);
} catch (err) {
  console.log('error');
}
```

## Good

```js
// Log with context
try {
  await saveToDatabase(record);
} catch (err) {
  console.error('Failed to save record:', { id: record.id, error: err.message });
  // Optionally rethrow
  throw err;
}

// Swallow intentionally with explicit logging and reason
try {
  await saveToDatabase(record);
} catch (err) {
  console.warn('Non-critical save failed (will retry):', err.message);
  retryQueue.push(record);
}

// Conditionally handle, always log the unexpected
try {
  await process(order);
} catch (err) {
  if (err instanceof ValidationError) {
    return res.status(400).json({ error: err.message });
  }
  // Unexpected error — log and rethrow
  console.error('Unexpected error processing order:', err);
  throw err;
}
```

## When Exceptions Apply

The only acceptable silent catch is for operations where failure is truly irrelevant and expected (e.g., non-critical analytics, optional cleanup). Even then, use a comment explaining why:

```js
try {
  await analytics.track(event);
} catch {
  // Analytics failure should never block the main flow
}
```

## See Also

- [err-no-swallow-rejection](./err-no-swallow-rejection.md) - Don't swallow in catch blocks
- [err-global-handlers](./err-global-handlers.md) - Global error handlers
