# lint-complexity-limit

> Limit cyclomatic complexity per function — ESLint `complexity`

## Why It Matters

High cyclomatic complexity (many branches, loops, conditions) makes functions hard to test, understand, and modify. A function with complexity 20 requires at least 20 test cases for full branch coverage. Limiting complexity forces extraction of helper functions, improving modularity and testability.

## Bad

```js
// Complexity ~15 — many interdependent conditions
function processOrder(order, user, config) {
  if (order.status === 'pending') {
    if (user.isPremium) {
      if (config.applyDiscount) {
        if (order.total > 100) {
          // ...
        } else if (order.total > 50) {
          // ...
        }
      }
    }
    if (order.items.length > 0) {
      for (const item of order.items) {
        if (item.inStock) {
          // ...
        } else {
          if (config.allowBackorder) {
            // ...
          }
        }
      }
    }
  }
}
```

## Good

```js
function processOrder(order, user, config) {
  if (order.status !== 'pending') return order;
  return processPendingOrder(order, user, config);
}

function processPendingOrder(order, user, config) {
  const withDiscount = applyDiscountIfEligible(order, user, config);
  const withInventory = processInventory(withDiscount, config);
  return withInventory;
}

function applyDiscountIfEligible(order, user, config) {
  if (!user.isPremium || !config.applyDiscount) return order;
  const rate = order.total > 100 ? 0.2 : order.total > 50 ? 0.1 : 0;
  return { ...order, total: order.total * (1 - rate) };
}
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'complexity': ['warn', 15],
  },
}
```

## When Exceptions Apply

State machines, parsers, and generated code often have high natural complexity. Use `eslint-disable` comments with justification for specific functions.

## See Also

- [proj-no-giant-files](./proj-no-giant-files.md) - File size limits
- [mod-separate-concerns](./mod-separate-concerns.md) - One module, one responsibility
