# err-result-pattern

> Use a `Result`-like return type for expected, recoverable failures

## Why It Matters

Throwing exceptions for failures that are a normal, expected part of a function's contract (a lookup that might miss, a parse that might fail, a network call that might time out) hides those outcomes from the type signature — callers only find out about them by reading the implementation or hitting an uncaught exception in production. A `Result<T, E>` return type makes "this can fail" part of the function's signature, and the compiler forces callers to handle both branches before accessing the success value.

## Bad

```typescript
function parsePort(raw: string): number {
  const port = Number(raw);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`invalid port: ${raw}`); // caller has no type-level hint this can throw
  }
  return port;
}

// Caller might easily forget the try/catch entirely
const port = parsePort(process.env.PORT ?? "");
startServer(port);
```

## Good

```typescript
type Result<T, E = Error> = { ok: true; value: T } | { ok: false; error: E };

function parsePort(raw: string): Result<number, string> {
  const port = Number(raw);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    return { ok: false, error: `invalid port: ${raw}` };
  }
  return { ok: true, value: port };
}

const result = parsePort(process.env.PORT ?? "");
if (!result.ok) {
  console.error(result.error);
  process.exit(1);
}
startServer(result.value); // compiler guarantees `value` exists here
```

## Helper Utilities

A small set of combinators keeps `Result` ergonomic instead of a pile of manual `if (!result.ok)` checks:

```typescript
function map<T, U, E>(result: Result<T, E>, fn: (value: T) => U): Result<U, E> {
  return result.ok ? { ok: true, value: fn(result.value) } : result;
}

function unwrapOr<T, E>(result: Result<T, E>, fallback: T): T {
  return result.ok ? result.value : fallback;
}

const doubled = map(parsePort("8080"), (p) => p * 2);
const safePort = unwrapOr(parsePort("nope"), 3000);
```

## When to Use `Result` vs `throw`

| Failure kind | Prefer |
|---|---|
| Expected, part of normal control flow (validation, parsing, lookup) | `Result<T, E>` |
| Unexpected, programmer error, or unrecoverable (bug, out-of-memory, invariant violation) | `throw` |
| Crossing an async boundary where rejection semantics are idiomatic | `throw` inside async, or `Result` wrapped in a resolved promise |

Libraries like `neverthrow` provide a battle-tested `Result` type with chaining (`.map`, `.andThen`, `.mapErr`) if you don't want to hand-roll the combinators.

## See Also

- [type-discriminated-union](type-discriminated-union.md) - Model variants with discriminated unions and a common tag field
- [err-async-propagation](err-async-propagation.md) - Let async/await propagate rejections naturally instead of mixing .then/.catch
- [err-custom-error-class](err-custom-error-class.md) - Extend Error with custom subclasses that carry structured context
