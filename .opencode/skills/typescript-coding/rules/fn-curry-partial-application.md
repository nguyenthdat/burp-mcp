# fn-curry-partial-application

> Use currying/partial application to produce reusable configured functions

## Why It Matters

When a function takes a "configuration" argument and a "data" argument, calling it repeatedly with the same configuration but different data (event handlers, array-method callbacks, repeated validation) leads to noisy, repetitive call sites. Currying — restructuring a function to take its arguments one at a time — lets you fix the configuration once and get back a specialized function that only needs the remaining, varying argument. This removes duplication and produces callbacks that slot directly into `map`/`filter`/event handlers without an inline arrow-wrapper at every call site.

## Bad

```typescript
function isWithinRange(min: number, max: number, value: number): boolean {
  return value >= min && value <= max;
}

const scores = [45, 92, 78, 61, 88];

// The same (min, max) pair repeated at every call site:
const passing = scores.filter((s) => isWithinRange(60, 100, s));
const failing = scores.filter((s) => !isWithinRange(60, 100, s));
const midRange = scores.filter((s) => isWithinRange(40, 70, s));
```

## Good

```typescript
function isWithinRange(min: number, max: number) {
  return (value: number): boolean => value >= min && value <= max;
}

const scores = [45, 92, 78, 61, 88];

const isPassing = isWithinRange(60, 100);   // configured once
const isMidRange = isWithinRange(40, 70);   // configured once

const passing = scores.filter(isPassing);   // clean callback, no wrapper arrow
const midRange = scores.filter(isMidRange);
```

## Generic Curry Helper

For functions with more arguments, a small generic `curry` utility (or a library) avoids hand-writing nested closures every time:

```typescript
function curry<A, B, C>(fn: (a: A, b: B) => C) {
  return (a: A) => (b: B) => fn(a, b);
}

const add = (a: number, b: number) => a + b;
const add10 = curry(add)(10);
add10(5); // 15
```

## Library Support

- **Ramda** — curries every function in its standard library by default (`R.filter`, `R.map`, `R.prop`, etc. are all auto-curried), designed around this style from the ground up.
- **lodash/fp** — the functional variant of lodash, with immutable, auto-curried, data-last versions of the standard lodash functions (`_.filter(predicate)(collection)` instead of `_.filter(collection, predicate)`).

```typescript
import { filter, propEq } from "ramda";

const activeUsers = filter(propEq("status", "active"));
activeUsers(users); // reusable, composable predicate
```

## When Not To Curry

Don't curry functions with only one argument (nothing to partially apply), or public APIs where curried multi-step calls (`f(a)(b)(c)`) would confuse consumers unfamiliar with the style. Reserve currying for internal helpers and predicate/callback factories where the reuse benefit is clear.

## See Also

- [fn-pipeline-composition](fn-pipeline-composition.md) - Compose sequential data transformations as an explicit pipeline
- [fn-pure-functions](fn-pure-functions.md) - Prefer pure functions with no hidden side effects
- [fn-array-methods-over-loops](fn-array-methods-over-loops.md) - Use `map`/`filter`/`reduce` for transformations instead of manual `for` loops
