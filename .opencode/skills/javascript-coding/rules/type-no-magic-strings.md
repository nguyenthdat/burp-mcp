# type-no-magic-strings

> Use constants or enum-like objects instead of magic strings

## Why It Matters

Magic strings scattered through code are a maintenance nightmare: typos aren't caught, there's no single place to see all valid values, and renaming requires find-and-replace across the entire codebase. Constants and enum-like objects centralize values, enable IDE autocompletion, and make valid values discoverable.

## Bad

```js
// Magic strings — fragile and unmaintainable
function processOrder(order) {
  if (order.status === 'pending') { /* ... */ }
  else if (order.status === 'processing') { /* ... */ }
  else if (order.status === 'shipped') { /* ... */ }
  else if (order.status === 'delivered') { /* ... */ }
}

// Typos silently fail
order.status = 'Pendng';  // Misspelled — no error

// String comparison scattered everywhere
if (user.role === 'admin') { /* ... */ }  // file A
if (userRole === 'admin') { /* ... */ }   // file B
if (role === 'administrator') { /* ... */ }  // file C — inconsistent!
```

## Good

```js
// Enum-like object — single source of truth
export const OrderStatus = {
  PENDING: 'pending',
  PROCESSING: 'processing',
  SHIPPED: 'shipped',
  DELIVERED: 'delivered',
  CANCELLED: 'cancelled',
};

function processOrder(order) {
  switch (order.status) {
    case OrderStatus.PENDING: return handlePending(order);
    case OrderStatus.PROCESSING: return handleProcessing(order);
    case OrderStatus.SHIPPED: return handleShipped(order);
    case OrderStatus.DELIVERED: return handleDelivered(order);
    default: throw new Error(`Unknown status: ${order.status}`);
  }
}
```

## String Union Validation

```js
// Define valid values
const VALID_ROLES = ['admin', 'user', 'moderator'];

function isValidRole(role) {
  return VALID_ROLES.includes(role);
}

// Or with a helper
const ROLES = Object.freeze({
  ADMIN: 'admin',
  USER: 'user',
  MODERATOR: 'moderator',
});

// Collect all valid values for validation
const ALL_ROLES = Object.values(ROLES);
```

## When Exceptions Apply

Configuration keys and simple one-off strings (error messages, log strings) don't need constants. Create constants when a string value is used in 3+ places or represents a domain concept.

## See Also

- [type-tagged-unions](./type-tagged-unions.md) - Discriminated unions with type field
- [name-UPPER_SNAKE](./name-UPPER_SNAKE.md) - UPPER_SNAKE_CASE for constants
