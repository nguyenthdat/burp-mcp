# err-no-swallow-rejection

> Don't catch errors without appropriate handling — always do something meaningful

## Why It Matters

Catching an error and doing nothing with it is functionally identical to ignoring the error entirely. If the error represents a genuine problem, swallowing it leads to data corruption, incomplete operations, and impossible-to-diagnose bugs. Every catch block must either recover, log, retry, or rethrow.

## Bad

```js
// Swallowing all errors — nothing happens on failure
try {
  await processOrder(order);
} catch (err) {
  // Nothing — order silently fails to process
}

// Catch-all with generic handler that doesn't distinguish error types
try {
  await doWork();
} catch (err) {
  console.error('Error');  // Logs but continues — state may be inconsistent
}
```

## Good

```js
// Recoverable: log and continue
try {
  await processOrder(order);
} catch (err) {
  if (err instanceof ValidationError) {
    orderQueue.push(order);  // Recover — queue for retry
    console.warn(`Order ${order.id} queued for retry:`, err.message);
  } else {
    throw err;  // Unexpected — rethrow
  }
}

// Rethrow with context
try {
  await saveRecord(record);
} catch (err) {
  throw new Error(`Failed to save record ${record.id}`, { cause: err });
}

// Recover with fallback
try {
  data = await fetchFromPrimary();
} catch (err) {
  console.warn('Primary source failed, using cache');
  data = await fetchFromCache();  // Graceful degradation
}
```

## Intentional Swallowing

```js
// Explicit and documented — this is fine
try {
  await nonCriticalAnalytics.track(event);
} catch {
  // Analytics must never block the main flow
}
```

## When Exceptions Apply

The only acceptable swallow is a documented, intentional decision where the failure mode is understood and the recovery strategy does not require the error data.

## See Also

- [err-avoid-silent-catch](./err-avoid-silent-catch.md) - Never silently swallow errors
- [err-global-handlers](./err-global-handlers.md) - Global unhandled error handlers
