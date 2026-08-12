# fn-array-methods-over-loops

> Use `map`/`filter`/`reduce` for transformations instead of manual `for` loops

## Why It Matters

A manual `for` loop that builds a new array mixes three concerns into one block: the iteration mechanics, the accumulator management, and the actual transformation logic. `map`/`filter`/`reduce` name the transformation's *intent* directly in the method name, so a reader sees "this filters" or "this transforms" before reading a single line of the body. This also removes an entire class of off-by-one and mutable-accumulator bugs (forgetting to push, double-pushing, wrong loop bounds) since the array methods handle iteration and accumulation correctly by construction.

## Bad

```typescript
function getActiveUserNames(users: User[]): string[] {
  const result: string[] = [];
  for (let i = 0; i < users.length; i++) {
    if (users[i].isActive) {
      result.push(users[i].name);
    }
  }
  return result;
}

function sumOrderTotals(orders: Order[]): number {
  let sum = 0;
  for (const order of orders) {
    sum = sum + order.total;
  }
  return sum;
}
```

## Good

```typescript
function getActiveUserNames(users: User[]): string[] {
  return users.filter((u) => u.isActive).map((u) => u.name);
}

function sumOrderTotals(orders: Order[]): number {
  return orders.reduce((sum, order) => sum + order.total, 0);
}
```

## Method-to-Intent Mapping

| Method | Use it to say... |
|---|---|
| `map` | "transform each element into something else" |
| `filter` | "keep only elements matching a condition" |
| `reduce` | "combine all elements into a single value" |
| `find` / `findIndex` | "locate the first matching element" |
| `some` / `every` | "check a condition across the collection" |
| `flatMap` | "transform and flatten one level" |

## When a for Loop Is Still Right

- Early exit is needed mid-iteration for performance (`for...of` with `break`), where `find`/`some` don't fit the exact shape of the work.
- The loop body performs `await` sequentially and the ordering must be preserved (`for await` or a manual `for` with `await` inside, rather than `map` + `Promise.all` if strict ordering/backpressure matters — see `async-for-await-iteration`).
- Extremely hot numeric loops where measured profiling shows the array-method call overhead matters (rare, and check before assuming).

## See Also

- [fn-avoid-reduce-abuse](fn-avoid-reduce-abuse.md) - Avoid `reduce` when a more specific method already expresses the intent
- [fn-avoid-side-effects-in-map](fn-avoid-side-effects-in-map.md) - Never use `.map()` purely for side effects; use `.forEach()` or a `for` loop
- [async-avoid-async-foreach](async-avoid-async-foreach.md) - `forEach` does not await its callback; use `for...of` for sequential async work
