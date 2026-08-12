# err-custom-error-classes

> Extend the Error class for domain-specific error types

## Why It Matters

Generic `Error` instances provide no way to distinguish between different failure modes programmatically. Custom error classes enable `instanceof` checks, carry domain-specific metadata (status codes, error codes), and make error handling precise. Throwing strings or plain objects loses the stack trace and makes debugging harder.

## Bad

```js
// Throwing strings — no stack trace, not an Error
if (!user) throw 'User not found';

// Throwing plain objects — not an Error
if (!valid) throw { code: 400, message: 'Invalid input' };

// Using generic Error with string matching
throw new Error('VALIDATION_ERROR: email is required');
// Caller must parse the message string
```

## Good

```js
class ValidationError extends Error {
  constructor(message, fields) {
    super(message);
    this.name = 'ValidationError';
    this.code = 'VALIDATION_ERROR';
    this.fields = fields;
  }
}

class NotFoundError extends Error {
  constructor(resource, id) {
    super(`${resource} with id ${id} not found`);
    this.name = 'NotFoundError';
    this.code = 'NOT_FOUND';
    this.statusCode = 404;
  }
}

// Usage
if (!user) throw new NotFoundError('User', userId);

// Caller uses instanceof
try {
  await process(input);
} catch (err) {
  if (err instanceof ValidationError) {
    return res.status(400).json({ error: err.message, fields: err.fields });
  }
  if (err instanceof NotFoundError) {
    return res.status(404).json({ error: err.message });
  }
  throw err;
}
```

## Error Hierarchy

```js
class AppError extends Error {
  constructor(message, { cause, code, statusCode } = {}) {
    super(message, { cause });
    this.name = this.constructor.name;
    this.code = code;
    this.statusCode = statusCode;
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
```

## When Exceptions Apply

For quick scripts and prototypes, `Error` is fine. Introduce custom error classes when the codebase grows beyond a single file.

## See Also

- [err-error-cause](./err-error-cause.md) - Chain errors with `{ cause }`
- [err-error-info](./err-error-info.md) - Attach metadata to errors
- [err-structured-error-handling](./err-structured-error-handling.md) - Error hierarchy design
