# perf-set-over-array-lookup

> Use `Set` and `Map` for O(1) lookups instead of `Array.includes()` or `Array.indexOf()` which are O(n)

## Why It Matters

`Array.includes()` and `Array.indexOf()` scan the array linearly. For large collections, this becomes a performance bottleneck — checking membership in a 100,000-element array does 100,000 comparisons. `Set.has()` and `Map.get()` use hash-based lookup (O(1)), making them dramatically faster for frequent lookups.

## Bad

```js
// O(n) lookup — slow for large arrays
const blockedIPs = ['1.2.3.4', '5.6.7.8', /* ... 50,000 entries */];

function isBlocked(ip) {
  return blockedIPs.includes(ip);  // 50,000 comparisons worst case
}

// O(n) deduplication
const unique = items.filter((item, index) =>
  items.indexOf(item) === index,
);
```

## Good

```js
// O(1) lookup — fast at any size
const blockedIPs = new Set(['1.2.3.4', '5.6.7.8', /* ... 50,000 entries */]);

function isBlocked(ip) {
  return blockedIPs.has(ip);  // O(1) — instant
}

// O(n) deduplication with O(1) checks
const unique = [...new Set(items)];

// Map for key-value lookups (faster than objects for dynamic keys)
const userCache = new Map();

function getUser(id) {
  if (userCache.has(id)) return userCache.get(id);
  const user = fetchUser(id);
  userCache.set(id, user);
  return user;
}
```

## When to Convert

```js
// Good: build Set once, look up many times
const allowedSet = new Set(allowedRoles);

function hasAccess(user) {
  return allowedSet.has(user.role);  // Fast
}

// Bad: convert to Set for a single lookup
function hasAccess(user) {
  return new Set(allowedRoles).has(user.role);  // Set creation is O(n) anyway
}
```

## When Exceptions Apply

For arrays with fewer than ~50 elements, the overhead of creating a Set outweighs the benefit. Profile before optimizing.

## See Also

- [fn-map-over-for](./fn-map-over-for.md) - Array method patterns
- [fn-groupBy-toMap](./fn-groupBy-toMap.md) - Map.groupBy()
