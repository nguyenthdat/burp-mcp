# fn-pipeline-composition

> Compose sequential data transformations as an explicit pipeline

## Why It Matters

A sequence of transformations written as deeply nested function calls (`h(g(f(x)))`) reads inside-out — the first thing that happens is the innermost call, which is the last thing your eye reaches. As more steps are added, the nesting becomes a wall of closing parentheses that's hard to scan and even harder to insert a new step into without renumbering the nesting. An explicit pipeline — whether via chained methods, a `pipe` helper, or intermediate named variables — reads top-to-bottom in the same order the data actually flows, and lets you add, remove, or reorder a step by touching one line.

## Bad

```typescript
function processOrder(rawOrder: RawOrder): Receipt {
  return formatReceipt(
    applyTax(
      applyDiscount(
        validateOrder(
          normalizeOrder(rawOrder)
        ),
        getActiveDiscount()
      ),
      getTaxRate()
    )
  );
}
// Reading order (innermost first) is the opposite of visual/reading order.
```

## Good

```typescript
function pipe<T>(...fns: Array<(arg: T) => T>) {
  return (input: T): T => fns.reduce((value, fn) => fn(value), input);
}

const processOrder = pipe<Order>(
  normalizeOrder,
  validateOrder,
  (order) => applyDiscount(order, getActiveDiscount()),
  (order) => applyTax(order, getTaxRate()),
);

const receipt = formatReceipt(processOrder(rawOrder));
// Reads top-to-bottom, in the exact order the data is transformed.
```

## Chained-Method Pipelines

When each step operates on the same collection type, method chaining is often the clearest pipeline of all:

```typescript
const topSpenders = orders
  .filter((o) => o.status === "completed")
  .map((o) => ({ userId: o.userId, amount: o.total }))
  .reduce(groupByUserTotal, new Map<string, number>());
```

## Native Pipe Operator (Proposal, Not Yet Standard)

A `|>` pipe operator is a TC39 proposal (still in draft as of 2025) that would let you write `x |> f |> g` directly. It is not yet part of the language or supported without a Babel plugin — use a `pipe`/`flow` helper (hand-rolled, or from a library) instead of waiting on it.

## Library Support

- **Ramda**'s `R.pipe(...)` / `R.compose(...)` — auto-curried functions compose naturally.
- **lodash/fp**'s `flow(...)` / `flowRight(...)` — same idea for lodash's data-last, curried functions.
- **RxJS**'s `pipe(...)` on Observables — the same left-to-right composition pattern applied to streams instead of plain values.

## See Also

- [fn-curry-partial-application](fn-curry-partial-application.md) - Use currying/partial application to produce reusable configured functions
- [fn-composition-over-inheritance](fn-composition-over-inheritance.md) - Compose small functions instead of building class inheritance hierarchies
- [fn-array-methods-over-loops](fn-array-methods-over-loops.md) - Use `map`/`filter`/`reduce` for transformations instead of manual `for` loops
