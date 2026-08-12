# err-cause-chaining

> Chain root causes with the standard `cause` option

## Why It Matters

When you catch a low-level error and throw a higher-level one to add context, discarding the original error destroys the information needed to actually debug the failure — you're left with "failed to save user" and no idea whether it was a network timeout, a constraint violation, or a serialization bug. The standard `cause` option (ES2022, supported since Node 16.9+ and all modern browsers) preserves the original error as a linked `cause` property, so logs and debuggers can walk the full chain.

## Bad

```typescript
async function saveUser(user: User) {
  try {
    await db.insert("users", user);
  } catch (err) {
    // Original error (constraint violation? connection drop?) is thrown away
    throw new Error("failed to save user");
  }
}
```

## Good

```typescript
async function saveUser(user: User) {
  try {
    await db.insert("users", user);
  } catch (err) {
    throw new Error("failed to save user", { cause: err });
  }
}

// Consuming the chain later, e.g. in a top-level logger:
try {
  await saveUser(user);
} catch (err) {
  if (err instanceof Error) {
    console.error(err.message);
    if (err.cause) {
      console.error("caused by:", err.cause);
    }
  }
}
```

## Printing the Full Chain

`console.error` in modern Node and browsers automatically prints nested `cause` chains, but when logging as structured JSON you often need to walk it manually:

```typescript
function serializeError(err: unknown): Record<string, unknown> {
  if (!(err instanceof Error)) {
    return { message: String(err) };
  }
  return {
    name: err.name,
    message: err.message,
    stack: err.stack,
    cause: err.cause ? serializeError(err.cause) : undefined,
  };
}
```

## Custom Errors and `cause`

Custom `Error` subclasses should forward `options` (including `cause`) to `super()` so they participate in the same chaining mechanism:

```typescript
class RepositoryError extends Error {
  constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "RepositoryError";
  }
}

throw new RepositoryError("failed to save user", { cause: err });
```

## See Also

- [err-rethrow-context](err-rethrow-context.md) - Add context when rethrowing instead of losing the original error
- [err-custom-error-class](err-custom-error-class.md) - Extend Error with custom subclasses that carry structured context
- [doc-throws-tag](doc-throws-tag.md) - Document thrown errors with the @throws TSDoc tag
