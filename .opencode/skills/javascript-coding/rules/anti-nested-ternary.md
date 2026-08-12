# anti-nested-ternary

> Don't nest ternary operators — use `if/else` or extract to a function

## Why It Matters

Nested ternaries sacrifice readability for brevity. The human brain parses `if/else` blocks effortlessly but struggles to track multiple `?` and `:` operators. What saves 5 lines of code costs 5 minutes of confusion for every reader. The code is written once but read hundreds of times.

## Bad

```js
// Nested ternary — a puzzle to decode
const status = order.isPaid
  ? order.isShipped
    ? 'completed'
    : 'processing'
  : order.isCancelled
    ? 'cancelled'
    : 'pending';

// Multi-level nesting in JSX
const label = isActive ? isAdmin ? 'Admin Active' : 'User Active' : 'Inactive';
```

## Good

```js
// if/else — instantly clear
let status;
if (order.isPaid) {
  status = order.isShipped ? 'completed' : 'processing';
} else {
  status = order.isCancelled ? 'cancelled' : 'pending';
}

// Or extract a function
function getOrderStatus(order) {
  if (!order.isPaid) return order.isCancelled ? 'cancelled' : 'pending';
  return order.isShipped ? 'completed' : 'processing';
}

// Object lookup table (cleanest for many cases)
const STATUS = {
  paid_shipped: 'completed',
  paid_pending: 'processing',
  unpaid_cancelled: 'cancelled',
  unpaid_active: 'pending',
};

const status = STATUS[`${order.paid ? 'paid' : 'unpaid'}_${getOrderState(order)}`];
```

## When Exceptions Apply

A single ternary (`a ? b : c`) is fine and often more readable than a 4-line if/else. The anti-pattern is specifically nesting (ternary inside ternary).

## See Also

- [anti-overly-smart-code](./anti-overly-smart-code.md) - Readability over cleverness
- [fn-pure-functions](./fn-pure-functions.md) - Extract to pure functions
