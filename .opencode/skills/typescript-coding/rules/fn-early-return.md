# fn-early-return

> Use early returns/guard clauses to reduce nesting

## Why It Matters

Each additional level of `if` nesting forces a reader to hold more conditions in their head simultaneously to understand what a given line of code assumes is true. A function that validates its way through several `if` blocks by nesting the "happy path" progressively deeper produces a triangular shape that's hard to scan and even harder to modify without breaking a sibling condition. Guard clauses that return (or throw) immediately on an invalid/edge case keep the main logic at a single indentation level, so the reader only has to track "what happens in the normal case" without also tracking every exclusion at once.

## Bad

```typescript
function calculateShipping(order: Order): number {
  if (order.items.length > 0) {
    if (order.destination) {
      if (order.destination.country === "US") {
        if (order.weight <= 50) {
          return order.weight * 0.5;
        } else {
          throw new Error("Package too heavy for domestic shipping");
        }
      } else {
        return order.weight * 1.2;
      }
    } else {
      throw new Error("Missing destination");
    }
  } else {
    throw new Error("Cannot ship an empty order");
  }
}
```

## Good

```typescript
function calculateShipping(order: Order): number {
  if (order.items.length === 0) {
    throw new Error("Cannot ship an empty order");
  }
  if (!order.destination) {
    throw new Error("Missing destination");
  }
  if (order.destination.country !== "US") {
    return order.weight * 1.2;
  }
  if (order.weight > 50) {
    throw new Error("Package too heavy for domestic shipping");
  }
  return order.weight * 0.5;
}
```

Every branch is handled once, at the top level, and the function's true "main path" (the last line) is immediately visible without unwinding nested braces.

## The Pattern

1. Identify every precondition/edge case that should short-circuit the function.
2. Handle each one with `if (condition) { return / throw / continue; }` as early as possible.
3. Let the remaining code assume all guard conditions have already passed — no more nested `if`/`else` needed for what's already been ruled out.

## Applies Inside Loops Too

```typescript
// Bad: nested happy path inside the loop
for (const user of users) {
  if (user.isActive) {
    if (user.email) {
      sendNotification(user);
    }
  }
}

// Good: guard clauses with `continue`
for (const user of users) {
  if (!user.isActive) continue;
  if (!user.email) continue;
  sendNotification(user);
}
```

## See Also

- [anti-deeply-nested-ternary](anti-deeply-nested-ternary.md) - Avoid nested ternaries; use if/else or a lookup for multi-branch logic
- [err-boundary-validation](err-boundary-validation.md) - Validate inputs at the boundary before business logic runs
- [type-exhaustive-switch](type-exhaustive-switch.md) - Ensure `switch` statements over unions handle every case exhaustively
