# perf-memoize

> Memoize expensive pure functions to avoid redundant computation

## Why It Matters

Pure functions with expensive computation (cryptography, parsing, complex calculations) called repeatedly with the same arguments waste CPU. Memoization caches results by input, returning O(1) for repeated calls. This is especially valuable for recursive functions with overlapping subproblems (dynamic programming) and for functions called in tight loops or render cycles.

## Bad

```js
// Recomputes for every call with the same input
function fibonacci(n) {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);  // O(2^n) — recomputes same values
}

function calculateTax(order) {
  // Complex calculation involving DB lookups, geolocation, etc.
  // Called thousands of times with the same order data
}
```

## Good

```js
// Memoized — O(n) for fibonacci
function memoize(fn) {
  const cache = new Map();
  return (arg) => {
    if (cache.has(arg)) return cache.get(arg);
    const result = fn(arg);
    cache.set(arg, result);
    return result;
  };
}

const fibonacci = memoize((n) => {
  if (n <= 1) return n;
  return fibonacci(n - 1) + fibonacci(n - 2);
});

// Multi-argument memoization
function memoizeMulti(fn) {
  const cache = new Map();
  return (...args) => {
    const key = JSON.stringify(args);
    if (cache.has(key)) return cache.get(key);
    const result = fn(...args);
    cache.set(key, result);
    return result;
  };
}

const calculateTax = memoizeMulti((orderId, region) => {
  // Expensive computation
});
```

## Memoization with TTL

```js
function memoizeWithTTL(fn, ttlMs) {
  const cache = new Map();

  return (...args) => {
    const key = JSON.stringify(args);
    const cached = cache.get(key);
    if (cached && Date.now() - cached.time < ttlMs) {
      return cached.value;
    }
    const value = fn(...args);
    cache.set(key, { value, time: Date.now() });
    return value;
  };
}
```

## When Exceptions Apply

Memoization adds memory overhead and only helps for functions called repeatedly with the same arguments. Don't memoize functions with side effects or functions that are rarely called.

## See Also

- [fn-pure-functions](./fn-pure-functions.md) - Pure functions
- [perf-object-pool](./perf-object-pool.md) - Object pooling
