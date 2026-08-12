# err-structured-error-handling

> Create an error hierarchy with instanceof checks for structured handling

## Why It Matters

Without a clear error hierarchy, error handling becomes a mess of string matching and brittle conditionals. A well-designed error taxonomy lets handlers at different levels of the call stack respond appropriately: HTTP middleware maps error types to status codes, services decide whether to retry, and callers decide whether to show an error to the user.

## Bad

```js
// String matching — fragile, misses typos
try {
  await process(data);
} catch (err) {
  if (err.message.includes('validation')) {
    return res.status(400).json({ error: err.message });
  }
  if (err.message.includes('not found')) {
    return res.status(404).json({ error: err.message });
  }
  return res.status(500).json({ error: 'Internal error' });
}
```

## Good

```js
class AppError extends Error {
  constructor(message, { cause, code, statusCode, retryable } = {}) {
    super(message, { cause });
    this.name = this.constructor.name;
    this.code = code;
    this.statusCode = statusCode ?? 500;
    this.retryable = retryable ?? false;
  }
}

class ValidationError extends AppError {
  constructor(message, fields) {
    super(message, { code: 'VALIDATION_ERROR', statusCode: 400 });
    this.fields = fields;
  }
}

class NotFoundError extends AppError {
  constructor(resource) {
    super(`${resource} not found`, { code: 'NOT_FOUND', statusCode: 404 });
  }
}

class DatabaseError extends AppError {
  constructor(message, cause) {
    super(message, { cause, code: 'DATABASE_ERROR', statusCode: 503, retryable: true });
  }
}

// Structured error handling middleware
function errorHandler(err, req, res, _next) {
  if (err instanceof ValidationError) {
    return res.status(400).json({ error: err.message, fields: err.fields });
  }
  if (err instanceof NotFoundError) {
    return res.status(404).json({ error: err.message });
  }
  if (err instanceof AppError) {
    return res.status(err.statusCode).json({ error: err.message });
  }
  console.error('Unhandled error:', err);
  res.status(500).json({ error: 'Internal server error' });
}
```

## When Exceptions Apply

For small projects (< 3 error types), a simple `Error` with a `code` string property is sufficient. Introduce a hierarchy when handler complexity grows.

## See Also

- [err-custom-error-classes](./err-custom-error-classes.md) - Custom error classes
- [err-error-info](./err-error-info.md) - Attach metadata to errors
