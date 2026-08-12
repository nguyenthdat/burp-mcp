# fn-avoid-reduce-abuse

> Avoid `reduce` when a more specific method already expresses the intent

## Why It Matters

`reduce` is the most general array method — it can implement `map`, `filter`, `find`, `some`, and `flatMap` — but generality comes at the cost of readability. When a reader sees `.reduce(...)`, they have to read the entire callback and the initial value to figure out what kind of operation is happening, whereas `.map(...)` or `.filter(...)` tells them immediately. Reaching for `reduce` by default, out of habit or a sense that it's "more functional," produces code that's objectively harder to review than the specific method that already exists for the job.

## Bad

```typescript
// This is just `.map()`, expressed as reduce
const doubled = numbers.reduce<number[]>((acc, n) => {
  acc.push(n * 2);
  return acc;
}, []);

// This is just `.filter()`, expressed as reduce
const evens = numbers.reduce<number[]>((acc, n) => {
  if (n % 2 === 0) acc.push(n);
  return acc;
}, []);

// This is just `.find()`, expressed as reduce (and it doesn't even short-circuit!)
const firstAdmin = users.reduce<User | undefined>((found, u) => {
  return found ?? (u.role === "admin" ? u : undefined);
}, undefined);

// This is just `.some()`, expressed as reduce
const hasNegative = numbers.reduce((found, n) => found || n < 0, false);
```

## Good

```typescript
const doubled = numbers.map((n) => n * 2);
const evens = numbers.filter((n) => n % 2 === 0);
const firstAdmin = users.find((u) => u.role === "admin"); // also short-circuits
const hasNegative = numbers.some((n) => n < 0); // also short-circuits
```

## When reduce Is Actually The Right Tool

`reduce` earns its place when the operation genuinely combines every element into a single accumulated value that isn't just "a filtered/mapped array" — cases with no dedicated method:

```typescript
// Summing / aggregating into a single number
const total = orders.reduce((sum, o) => sum + o.amount, 0);

// Building a lookup map/dictionary from an array
const byId = users.reduce<Record<string, User>>((acc, u) => {
  acc[u.id] = u;
  return acc;
}, {});

// Grouping (until Object.groupBy/Map.groupBy is available on your target runtime)
const byRole = users.reduce<Record<string, User[]>>((acc, u) => {
  (acc[u.role] ??= []).push(u);
  return acc;
}, {});
```

## A Quick Test

Before reaching for `reduce`, ask: "is this producing a new array of the same length via a per-element transform, a subset via a per-element test, or a single boolean/element?" If yes, there is almost certainly a named method that says so more clearly.

## See Also

- [fn-array-methods-over-loops](fn-array-methods-over-loops.md) - Use `map`/`filter`/`reduce` for transformations instead of manual `for` loops
- [fn-avoid-side-effects-in-map](fn-avoid-side-effects-in-map.md) - Never use `.map()` purely for side effects; use `.forEach()` or a `for` loop
- [name-verb-noun-functions](name-verb-noun-functions.md) - Name functions with a leading verb describing the action they perform
