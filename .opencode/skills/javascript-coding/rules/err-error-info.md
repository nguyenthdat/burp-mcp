# err-error-info

> Attach useful metadata (code, statusCode, details) to custom errors

## Why It Matters

A plain `Error` with just a message forces callers to parse strings or make assumptions about the error. Attaching structured metadata (`code`, `statusCode`, `details`) allows programmatic handling: HTTP middleware can map codes to statuses, logging systems can group by code, and retry logic can decide based on `retryable`. Make errors machine-readable.

## Bad

```js
// No metadata — callers must parse the message
throw new Error('Email is required');

// Inconsistent property names
throw Object.assign(new Error('Not found'), { code: 404, httpStatus: 404 });
throw Object.assign(new Error('Invalid'), { errorCode: 'VALIDATION' });
```

## Good

```js
class AppError extends Error {
  constructor(message, { code, statusCode, details, retryable } = {}) {
    super(message);
    this.name = this.constructor.name;
    this.code = code ?? 'INTERNAL_ERROR';
    this.statusCode = statusCode ?? 500;
    this.details = details ?? {};
    this.retryable = retryable ?? false;
  }

  toJSON() {
    return {
      error: this.message,
      code: this.code,
      details: this.details,
    };
  }
}

// Usage
throw new AppError('Email is required', {
  code: 'VALIDATION_ERROR',
  statusCode: 400,
  details: { field: 'email' },
});

throw new AppError('Database connection lost', {
  code: 'DATABASE_UNAVAILABLE',
  statusCode: 503,
  retryable: true,
});

// Caller can programmatically handle
try {
  await doWork();
} catch (err) {
  if (err.code === 'DATABASE_UNAVAILABLE' && err.retryable) {
    await retryWithBackoff(doWork);
  }
  if (err.statusCode) {
    res.status(err.statusCode).json(err.toJSON());
  }
}
```

## When Exceptions Apply

For quick scripts, a plain Error with a descriptive message is fine. Add structured metadata when the error crosses API boundaries or needs programmatic handling.

## See Also

- [err-custom-error-classes](./err-custom-error-classes.md) - Custom error classes
- [err-structured-error-handling](./err-structured-error-handling.md) - Error hierarchy design
