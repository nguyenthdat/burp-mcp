# fn-pure-functions

> Write pure functions that don't mutate inputs or depend on external state

## Why It Matters

Pure functions — same input always produces same output, no side effects — are the easiest to test, debug, and reason about. They're inherently reusable and composable. Impure functions that mutate arguments or depend on global state create hidden coupling and make tests unreliable.

## Bad

```js
// Impure — mutates input
function addTax(order) {
  order.total = order.subtotal * 1.10;  // Mutates the order object
  return order;
}

// Impure — depends on global state
let taxRate = 0.10;
function calculateTax(subtotal) {
  return subtotal * taxRate;  // Result depends on mutable global
}

// Impure — side effect (logging)
function processOrder(order) {
  console.log('Processing', order.id);  // Side effect
  return saveToDatabase(order);
}
```

## Good

```js
// Pure — returns new object, doesn't mutate input
function addTax(order) {
  return { ...order, total: order.subtotal * 1.10 };
}

// Pure — receives all dependencies as arguments
function calculateTax(subtotal, taxRate) {
  return subtotal * taxRate;
}

// Acceptable impurity at I/O boundary
async function processOrder(order) {
  const enriched = enrichOrder(order);   // Pure transformation
  return await saveToDatabase(enriched); // I/O side effect at boundary
}
```

## Benefits of Pure Functions

```js
// Easy to test
const result = calculateTax(100, 0.10);
assert.strictEqual(result, 110);

// Safe to memoize
import { memoize } from './memoize.js';
const cachedTax = memoize(calculateTax);

// Safe to compose
const withTax = (subtotal) => calculateTax(subtotal, 0.10);
const withDiscount = (subtotal) => subtotal * 0.9;
const final = withTax(withDiscount(100));  // 99
```

## When Exceptions Apply

Pure functions are an ideal, not an absolute. I/O operations (file, network, database) are inherently impure. Isolate impure code at the application boundary and keep the core logic pure.

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable data patterns
- [perf-memoize](./perf-memoize.md) - Memoize expensive pure functions
