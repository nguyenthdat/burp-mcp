# err-no-throw-string

> Always throw Error instances, never strings or primitives

## Why It Matters

Throwing a string or primitive value bypasses JavaScript's error infrastructure — no stack trace is captured, no `instanceof Error` checks work, and error-reporting tools can't extract useful diagnostics. Always throw an instance of `Error` (or a subclass) so that the runtime can capture the call stack.

## Bad

```js
// String — no stack trace
if (!user) throw 'User not found';

// Number — no stack trace
if (count < 0) throw -1;

// Plain object — no .stack property
if (!valid) throw { message: 'Invalid', code: 400 };

// null / undefined
if (!config) throw null;
```

## Good

```js
// Always throw Error instances
if (!user) throw new Error('User not found');

// Use custom error classes for structured handling
if (!user) throw new NotFoundError('User', userId);

// Include relevant context
if (count < 0) throw new RangeError(`Count must be non-negative, got ${count}`);

if (!valid) throw new ValidationError('Invalid input', errors);
```

## Standard Error Subclasses

```js
// Use built-in error types when appropriate
throw new TypeError('value must be a string');
throw new RangeError('index out of bounds');
throw new SyntaxError('unexpected token');
throw new ReferenceError('x is not defined');

// Throw AssertionError from node:assert for invariants
import assert from 'node:assert';

function divide(a, b) {
  assert(b !== 0, 'Division by zero');
  return a / b;
}
```

## When Exceptions Apply

There are no exceptions. Always throw `Error` instances. Linters like `no-throw-literal` enforce this automatically.

## See Also

- [err-custom-error-classes](./err-custom-error-classes.md) - Domain-specific error types
- [anti-throw-non-error](./anti-throw-non-error.md) - Anti-pattern of throwing non-errors
