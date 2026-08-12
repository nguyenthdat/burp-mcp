# err-assertion-libraries

> Use assertion for input validation at API boundaries with assertion libraries

## Why It Matters

JavaScript doesn't have built-in runtime type checking. Invalid inputs cause cryptic errors deep in the call stack. Assertion libraries validate inputs at function boundaries, failing fast with clear messages. This prevents corrupt data from propagating and simplifies debugging.

## Bad

```js
// No validation — cryptic error later
function calculateTax(price, rate) {
  return price * rate;  // price='abc' → NaN
}

// Manual validation — verbose, inconsistent
function calculateTax(price, rate) {
  if (typeof price !== 'number') throw new Error('price must be a number');
  if (typeof rate !== 'number') throw new Error('rate must be a number');
  if (price < 0) throw new Error('price must be non-negative');
  return price * rate;
}
```

## Good

```js
import assert from 'node:assert/strict';

function calculateTax(price, rate) {
  assert.strictEqual(typeof price, 'number', 'price must be a number');
  assert.strictEqual(typeof rate, 'number', 'rate must be a number');
  assert.ok(price >= 0, 'price must be non-negative');
  assert.ok(rate >= 0 && rate <= 1, 'rate must be between 0 and 1');
  return price * rate;
}

// With a validation library for complex shapes
import { z } from 'zod';

const OrderSchema = z.object({
  userId: z.string().uuid(),
  items: z.array(z.object({
    productId: z.string(),
    quantity: z.number().int().positive(),
  })).min(1),
  total: z.number().positive(),
});

function processOrder(input) {
  const order = OrderSchema.parse(input);  // Throws ZodError with details
  // order is now typed and validated
}
```

## When Exceptions Apply

Assertions have runtime cost. In hot loops, consider whether the input can be trusted and skip validation if callers are guaranteed to be correct (e.g., internal functions with well-typed callers).

## See Also

- [err-type-check-inputs](./err-type-check-inputs.md) - Validate at function boundaries
- [type-zod-validation](./type-zod-validation.md) - Use zod/joi for API input
