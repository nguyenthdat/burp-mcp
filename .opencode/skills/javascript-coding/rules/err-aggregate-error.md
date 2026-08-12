# err-aggregate-error

> Use AggregateError for operations that produce multiple errors

## Why It Matters

When a single operation can fail in multiple ways simultaneously (e.g., validating a form, fetching multiple resources, running parallel tasks), throwing just one error loses information. `AggregateError` collects all errors so the caller can inspect each one and decide how to respond.

## Bad

```js
// Only reports the first error found
function validateForm(data) {
  const errors = [];
  if (!data.name) errors.push(new Error('Name is required'));
  if (!data.email) errors.push(new Error('Email is required'));
  if (data.age < 18) errors.push(new Error('Must be 18+'));
  if (errors.length > 0) throw errors[0];  // Other errors lost
}

// Promise.any discards all rejection reasons
try {
  await Promise.any(promises);
} catch (err) {
  // err is AggregateError — but we don't use it
  console.error('All failed:', err);
}
```

## Good

```js
function validateForm(data) {
  const errors = [];
  if (!data.name) errors.push(new ValidationError('Name is required'));
  if (!data.email) errors.push(new ValidationError('Email is required'));
  if (data.age < 18) errors.push(new ValidationError('Must be 18+'));
  if (errors.length > 0) throw new AggregateError(errors, 'Form validation failed');
}

// Caller inspects all errors
try {
  validateForm(data);
} catch (err) {
  if (err instanceof AggregateError) {
    for (const e of err.errors) {
      console.error(`- ${e.message}`);
    }
    return res.status(400).json({
      error: err.message,
      details: err.errors.map(e => e.message),
    });
  }
}

// Promise.any with AggregateError inspection
try {
  const result = await Promise.any([
    fetchFromCache(),
    fetchFromPrimary(),
    fetchFromFallback(),
  ]);
} catch (err) {
  console.error('All sources failed:');
  for (const e of err.errors) {
    console.error(`  ${e.message}`);
  }
}
```

## When Exceptions Apply

Use `AggregateError` when you need the caller to see all individual failures. Use a single error when only one failure path is expected at a time.

## See Also

- [async-promise-allsettled](./async-promise-allsettled.md) - allSettled for partial failures
- [err-structured-error-handling](./err-structured-error-handling.md) - Error hierarchy
