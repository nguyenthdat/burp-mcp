# anti-delete-operator

> Don't use `delete` on arrays or in hot paths — it deoptimizes objects and leaves array holes

## Why It Matters

The `delete` operator removes a property from an object, changing its "hidden class" in V8 and deoptimizing property access. On arrays, `delete` creates a hole (empty slot) without changing `length`, causing confusing iteration behavior. Use `splice`, `filter`, or set to `undefined` instead.

## Bad

```js
// delete on array — leaves hole, doesn't change length
const arr = [1, 2, 3];
delete arr[1];
arr.length;  // 3 — still!
console.log(arr);  // [1, empty, 3]
arr.forEach(item => console.log(item));  // Skips index 1

// delete on object in hot loop — deoptimizes
for (let i = 0; i < 100000; i++) {
  const obj = { a: 1, b: 2, c: 3 };
  delete obj.b;  // Changes hidden class 100k times
}
```

## Good

```js
// Arrays: use splice or filter
arr.splice(1, 1);                    // Removes element, shifts rest
const filtered = arr.filter((_, i) => i !== 1);  // Immutable

// Objects: create new without the property
const { b, ...rest } = obj;  // rest is { a: 1, c: 3 }

// If you must mutate, set to undefined
obj.b = undefined;

// Use Map for dynamic key removal
const map = new Map(Object.entries(obj));
map.delete('b');  // Optimized for deletion
```

## When Exceptions Apply

`delete` is acceptable for infrequent operations on small objects. Avoid in loops and hot paths.

## See Also

- [fn-avoid-delete-mutation](./fn-avoid-delete-mutation.md) - Avoid delete mutation
- [fn-immutability](./fn-immutability.md) - Immutable patterns
