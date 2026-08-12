# anti-in-operator-array

> Don't use `in` to iterate arrays — use `for...of`, `forEach()`, or a classic `for` loop

## Why It Matters

The `in` operator checks property existence in the prototype chain, not values. `for...in` iterates over enumerable properties (including prototype methods), not array elements, and the order is not guaranteed. Using it on arrays produces indices as strings, skips empty slots unpredictably, and includes inherited properties.

## Bad

```js
const arr = [10, 20, 30];

// for...in iterates keys (strings), not values
for (const i in arr) {
  console.log(arr[i]);  // Works but i is '0', '1', '2' (strings)
}

// If Array.prototype is extended, for...in picks it up
Array.prototype.custom = 'hack';
for (const key in arr) {
  console.log(key);  // '0', '1', '2', 'custom' — surprise!
}
```

## Good

```js
const arr = [10, 20, 30];

// for...of — iterates values
for (const value of arr) {
  console.log(value);  // 10, 20, 30
}

// forEach — values with index
arr.forEach((value, index) => {
  console.log(index, value);
});

// Classic for — indices as numbers
for (let i = 0; i < arr.length; i++) {
  console.log(arr[i]);
}

// for...in is for objects, not arrays
const obj = { a: 1, b: 2 };
for (const key in obj) {
  if (Object.hasOwn(obj, key)) {
    console.log(key, obj[key]);
  }
}
```

## When Exceptions Apply

`for...in` is appropriate for iterating over object properties (with `hasOwn` check). It's never appropriate for arrays.

## See Also

- [perf-for-of-array](./perf-for-of-array.md) - for...of for arrays
- [fn-map-over-for](./fn-map-over-for.md) - Array methods
