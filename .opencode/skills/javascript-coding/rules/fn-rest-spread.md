# fn-rest-spread

> Use rest parameters (`...args`) over the `arguments` object; use spread (`...`) for array/object manipulation

## Why It Matters

The `arguments` object is array-like but not an array, doesn't work in arrow functions, and obscures the function's parameter list. Rest parameters produce a real array and work in all function types. The spread operator replaces verbose `.apply()`, `.concat()`, and `Object.assign()` patterns with a single, readable operator.

## Bad

```js
// arguments object — not available in arrow functions, not a real array
function logAll() {
  const args = Array.prototype.slice.call(arguments);  // Ugh
  args.forEach(arg => console.log(arg));
}

// Manual concat and apply
const combined = [1, 2].concat(more);
const max = Math.max.apply(null, numbers);

// Object.assign for merging
const merged = Object.assign({}, defaults, overrides);
```

## Good

```js
// Rest parameters — real array, works everywhere
function logAll(...args) {
  args.forEach(arg => console.log(arg));
}

// Spread for arrays
const combined = [1, 2, ...more];
const max = Math.max(...numbers);

// Spread for objects
const merged = { ...defaults, ...overrides };

// Spread for function calls
fetch(url, { ...defaultHeaders, ...customHeaders });
```

## Rest in Destructuring

```js
const [first, second, ...rest] = items;
const { id, ...data } = record;

function processUser({ name, email, ...rest }) {
  // name, email are extracted; rest contains remaining properties
}
```

## When Exceptions Apply

Very large arrays with spread can cause stack overflow or performance issues. For arrays with > 100,000 elements, use `.push(...items)` or `Array.prototype.push.apply` instead of spread:

```js
// Better for huge arrays
target.push(...source);  // Acceptable up to ~125k elements in V8
```

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable patterns with spread
- [fn-destructure](./fn-destructure.md) - Destructuring patterns
- [perf-array-push-spread](./perf-array-push-spread.md) - push vs concat performance
