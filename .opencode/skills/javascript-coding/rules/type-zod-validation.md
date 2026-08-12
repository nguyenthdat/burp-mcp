# type-zod-validation

> Use schema validation libraries (zod, joi, valibot) for validating API input and external data

## Why It Matters

Manual validation with `typeof` checks becomes unwieldy for complex nested objects. Schema validation libraries provide declarative, composable schemas that validate, parse, and provide TypeScript-like type inference. They catch invalid shapes at the boundary and provide detailed error messages for every field.

## Bad

```js
// Manual validation — verbose, incomplete, hard to maintain
function validateOrder(data) {
  if (!data || typeof data !== 'object') throw new Error('Invalid order');
  if (typeof data.userId !== 'string') throw new Error('Invalid userId');
  if (!Array.isArray(data.items)) throw new Error('Items must be an array');
  if (data.items.length === 0) throw new Error('Order must have items');

  for (const item of data.items) {
    if (typeof item.productId !== 'string') throw new Error('Invalid productId');
    if (typeof item.quantity !== 'number' || item.quantity <= 0) throw new Error('Invalid quantity');
  }

  if (data.total && typeof data.total !== 'number') throw new Error('Invalid total');
  return data;
}
```

## Good

```js
import { z } from 'zod';

const OrderItemSchema = z.object({
  productId: z.string().min(1),
  quantity: z.number().int().positive(),
});

const OrderSchema = z.object({
  userId: z.string().uuid(),
  items: z.array(OrderItemSchema).min(1, 'Order must have at least one item'),
  total: z.number().positive().optional(),
  notes: z.string().max(500).optional(),
});

function validateOrder(input) {
  return OrderSchema.parse(input);  // Throws ZodError with all field errors
}

// Safe parsing (returns result object instead of throwing)
const result = OrderSchema.safeParse(input);
if (!result.success) {
  console.error(result.error.issues);
  return { error: 'Validation failed', details: result.error.issues };
}
```

## Transform and Refine

```js
const UserSchema = z.object({
  email: z.string().email().transform(email => email.toLowerCase()),
  age: z.number().refine(age => age >= 18, 'Must be 18 or older'),
  password: z.string().min(8).refine(
    pw => /[A-Z]/.test(pw) && /[0-9]/.test(pw),
    'Password must contain uppercase and number',
  ),
});
```

## When Exceptions Apply

For very simple validation (1-2 fields) or performance-critical hot paths, manual checks may be more efficient. Use zod for API boundaries, form validation, and configuration parsing.

## See Also

- [err-assertion-libraries](./err-assertion-libraries.md) - Assertion for validation
- [type-parse-dont-assume](./type-parse-dont-assume.md) - Parse with schema validation
- [type-validate-config](./type-validate-config.md) - Validate config at startup
