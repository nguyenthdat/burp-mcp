# err-typed-catch-unknown

> Type the catch binding as `unknown` and narrow before use

## Why It Matters

JavaScript allows any value at all to be thrown — not just `Error` instances, but strings, numbers, or plain objects, whether from your own code, a dependency, or a runtime error. Modern TypeScript (with `useUnknownInCatchVariables`, on by default under `strict` since TS 4.4) types the catch binding as `unknown` rather than `any` specifically to stop you from calling `.message` or `.stack` on it without checking first — those properties simply don't exist on a thrown string or plain object, and accessing them crashes.

## Bad

```typescript
// Pre-TS 4.4 behavior, or with useUnknownInCatchVariables disabled
try {
  await riskyOperation();
} catch (err) {
  // err implicitly typed `any` — compiles, but crashes if something non-Error was thrown
  console.error(err.message);
  logger.error(err.stack);
}
```

## Good

```typescript
try {
  await riskyOperation();
} catch (err: unknown) {
  if (err instanceof Error) {
    console.error(err.message);
    logger.error(err.stack);
  } else {
    // Handles the case where a non-Error value was thrown (string, plain object, etc.)
    console.error("non-Error thrown:", String(err));
  }
}
```

## A Reusable Normalizer

Since this pattern repeats at every catch site, centralize it once:

```typescript
function toError(value: unknown): Error {
  if (value instanceof Error) {
    return value;
  }
  if (typeof value === "string") {
    return new Error(value);
  }
  return new Error(`unknown error: ${JSON.stringify(value)}`);
}

try {
  await riskyOperation();
} catch (err) {
  const normalized = toError(err);
  logger.error(normalized.message, { stack: normalized.stack });
}
```

## Configuration

```json
{
  "compilerOptions": {
    "strict": true,
    "useUnknownInCatchVariables": true
  }
}
```

This is included automatically under `strict: true` in modern TypeScript, but it's worth confirming explicitly when auditing a `tsconfig.json` that predates TS 4.4 or that overrides individual strict flags.

## See Also

- [type-unknown-over-any](type-unknown-over-any.md) - Use unknown instead of any for values of uncertain type
- [err-specific-catch](err-specific-catch.md) - Catch and handle specific error types instead of a blanket catch-all
- [err-no-throw-strings](err-no-throw-strings.md) - Always throw Error instances, never strings or plain objects
