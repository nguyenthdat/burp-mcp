# err-custom-error-class

> Extend `Error` with custom subclasses that carry structured context

## Why It Matters

Throwing a generic `Error` with only a message string forces every caller to parse text to figure out what went wrong, which is brittle and breaks the moment the wording changes. A custom `Error` subclass lets you attach structured, typed fields (an HTTP status, a resource id, a validation issue list) and lets callers distinguish failure kinds with `instanceof`, enabling precise handling instead of string matching.

## Bad

```typescript
function fetchUser(id: string) {
  const user = db.find(id);
  if (!user) {
    throw new Error(`User ${id} not found`); // caller must parse the message string
  }
  return user;
}

try {
  fetchUser("123");
} catch (e: unknown) {
  if (e instanceof Error && e.message.includes("not found")) {
    // Fragile: breaks if the message wording ever changes
    respondNotFound();
  }
}
```

## Good

```typescript
class NotFoundError extends Error {
  readonly resource: string;
  readonly id: string;

  constructor(resource: string, id: string) {
    super(`${resource} not found: ${id}`);
    this.name = "NotFoundError";
    this.resource = resource;
    this.id = id;
    // Required for correct prototype chain when targeting ES5/downleveled classes
    Object.setPrototypeOf(this, NotFoundError.prototype);
  }
}

function fetchUser(id: string) {
  const user = db.find(id);
  if (!user) {
    throw new NotFoundError("User", id);
  }
  return user;
}

try {
  fetchUser("123");
} catch (e: unknown) {
  if (e instanceof NotFoundError) {
    respondNotFound(e.resource, e.id); // structured, typed access — no string parsing
  } else {
    throw e;
  }
}
```

## A Small Error Hierarchy

```typescript
abstract class AppError extends Error {
  abstract readonly code: string;

  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

class NotFoundError extends AppError {
  readonly code = "NOT_FOUND";
  constructor(public readonly resource: string, public readonly id: string) {
    super(`${resource} not found: ${id}`);
  }
}

class ValidationError extends AppError {
  readonly code = "VALIDATION_ERROR";
  constructor(public readonly issues: string[]) {
    super(`validation failed: ${issues.join(", ")}`);
  }
}
```

A shared base class lets top-level error handlers map `error.code` to an HTTP status or log severity in one place, instead of a chain of `instanceof` checks.

## See Also

- [err-cause-chaining](err-cause-chaining.md) - Chain root causes with the standard cause option
- [err-specific-catch](err-specific-catch.md) - Catch and handle specific error types instead of a blanket catch-all
- [err-no-throw-strings](err-no-throw-strings.md) - Always throw Error instances, never strings or plain objects
