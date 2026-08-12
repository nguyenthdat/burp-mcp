# lint-no-param-reassign

> Don't reassign function parameters — ESLint `no-param-reassign`

## Why It Matters

Reassigning function parameters makes the function's behavior harder to reason about and can cause bugs when the original value (passed by reference) is unexpectedly modified. It also prevents JavaScript engine optimizations. Use local variables instead of mutating parameters.

## Bad

```js
function processOrder(order) {
  order.status = 'processing';  // Mutates the caller's object

  if (!order.discount) {
    order.total = order.subtotal;  // Mutates
  }

  order = enrichOrder(order);  // Reassigns parameter
  return order;
}
```

## Good

```js
function processOrder(order) {
  const status = 'processing';
  const total = order.discount ? calculateDiscount(order) : order.subtotal;
  const enriched = enrichOrder(order);

  return { ...order, status, total, ...enriched };
}
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'no-param-reassign': ['error', {
      props: true,
      ignorePropertyModificationsFor: ['acc', 'req', 'res'],
    }],
  },
}
```

## When Exceptions Apply

Redux-style reducers and `Array.reduce()` accumulators need to modify their state parameter. ESLint allows whitelisting specific parameter names (`acc`, `state`).

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable patterns
- [fn-pure-functions](./fn-pure-functions.md) - Pure functions
