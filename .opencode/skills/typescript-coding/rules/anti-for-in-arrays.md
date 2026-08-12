# anti-for-in-arrays

> Don't use `for...in` to iterate arrays

## Why It Matters

`for...in` enumerates all *enumerable* property keys on an object, including inherited ones and any custom properties attached to an array — it does not guarantee numeric order across engines, and it returns keys as strings, not numbers. On an array, that means `for (const i in arr)` can iterate extra, unexpected keys (e.g., a property added by a library that patches `Array.prototype` or attaches metadata directly onto the array instance), and `i` is a string, so `arr[i] + 1` silently does string concatenation instead of numeric addition if you're not careful. Arrays have purpose-built iteration constructs (`for...of`, `.forEach`, `.map`) that iterate only the array's elements, in order, as the correct type — `for...in` exists for plain objects, not arrays.

## Bad

```typescript
const scores = [10, 20, 30];
(scores as any).total = 60; // an extra property attached to the array

for (const i in scores) {
  console.log(scores[i]); // also logs `60` — the extra property — not just elements
  console.log(typeof i); // "string", not "number" — easy to misuse in arithmetic
}
```

## Good

```typescript
const scores = [10, 20, 30];

for (const score of scores) {
  console.log(score); // iterates only the actual elements, correctly typed as number
}

// Need the index too? for...of with entries()
for (const [index, score] of scores.entries()) {
  console.log(index, score);
}

// Transformation: prefer array methods over manual loops
const doubled = scores.map((score) => score * 2);
```

## When You Do Need Object Keys

`for...in` is appropriate for plain objects when you genuinely want enumerable keys (including inherited ones, which is rarely desired) — but even there, `Object.keys()`/`Object.entries()` combined with `for...of` are usually clearer because they don't traverse the prototype chain:

```typescript
const config = { host: 'localhost', port: 8080 };

for (const [key, value] of Object.entries(config)) {
  console.log(key, value);
}
```

## See Also

- [fn-array-methods-over-loops](fn-array-methods-over-loops.md) - Prefer array methods (`map`/`filter`/`reduce`) over manual loops
- [async-for-await-iteration](async-for-await-iteration.md) - Use `for await...of` to consume async iterables
- [imm-avoid-array-mutation](imm-avoid-array-mutation.md) - Avoid mutating arrays in place
