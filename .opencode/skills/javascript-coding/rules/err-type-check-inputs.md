# err-type-check-inputs

> Validate types at function boundaries before performing operations

## Why It Matters

JavaScript's dynamic typing means invalid inputs are not caught at call time — they cause cryptic errors deep within the function, often in a different stack frame. Type-checking at the function boundary fails fast where the mistake was made, providing clear error messages and preventing data corruption.

## Bad

```js
// Type mismatch causes confusing error deep in the function
function formatName(user) {
  return `${user.first.toUpperCase()} ${user.last.toUpperCase()}`;
}

formatName(null);        // TypeError: Cannot read properties of null
formatName({ first: 'a' });  // "A UNDEFINED"
formatName({ first: 1 });    // TypeError: user.first.toUpperCase is not a function
```

## Good

```js
function formatName(user) {
  if (typeof user !== 'object' || user === null) {
    throw new TypeError('user must be an object');
  }
  if (typeof user.first !== 'string') {
    throw new TypeError('user.first must be a string');
  }
  if (typeof user.last !== 'string') {
    throw new TypeError('user.last must be a string');
  }
  return `${user.first.toUpperCase()} ${user.last.toUpperCase()}`;
}

// Using a validation library
import { z } from 'zod';

const UserSchema = z.object({
  first: z.string(),
  last: z.string(),
});

function formatName(input) {
  const user = UserSchema.parse(input);
  return `${user.first.toUpperCase()} ${user.last.toUpperCase()}`;
}
```

## Guard Clauses Pattern

```js
function processPayment(amount, source, destination) {
  if (typeof amount !== 'number' || amount <= 0) {
    throw new TypeError('amount must be a positive number');
  }
  if (typeof source !== 'string' || source.length === 0) {
    throw new TypeError('source must be a non-empty string');
  }
  if (typeof destination !== 'string' || destination.length === 0) {
    throw new TypeError('destination must be a non-empty string');
  }

  // Actual logic — all inputs are validated
  return transfer(amount, source, destination);
}
```

## When Exceptions Apply

Internal helper functions called with predictable inputs can skip validation. Reserve boundary checks for public API functions, HTTP handlers, and library exports.

## See Also

- [err-assertion-libraries](./err-assertion-libraries.md) - Use assertion libraries
- [type-typeof-guards](./type-typeof-guards.md) - Type guards at public boundaries
