# perf-array-push-spread

> Use `.push(...items)` over `.concat()` for adding multiple items to large arrays

## Why It Matters

`.concat()` creates a new array, copying all existing elements. For large arrays, this doubles memory usage temporarily and takes O(n+m) time. `.push()` appends in-place without copying the original array. When you don't need a new array (mutating is acceptable), `.push()` is significantly more efficient.

## Bad

```js
// concat creates a new array — doubles memory
let items = [/* 100,000 elements */];
items = items.concat(newItems);  // Allocates new 100k+ array

// Spread in array literal also creates a new array
items = [...items, ...newItems];  // Same allocation
```

## Good

```js
// push mutates in place — no extra allocation
items.push(...newItems);

// For truly massive arrays, push elements individually
// (spread has argument count limits)
for (const item of newItems) {
  items.push(item);
}

// If you need immutability, create the new array explicitly
const newArray = [...items, ...newItems];  // OK if immutability is required
```

## When Exceptions Apply

When you need an immutable update (e.g., Redux-style state management), creating a new array with spread is the correct approach. The performance cost is acceptable for correctness.

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable patterns
- [fn-rest-spread](./fn-rest-spread.md) - Rest/spread operators
