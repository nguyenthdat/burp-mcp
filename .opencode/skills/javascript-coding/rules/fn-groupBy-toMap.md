# fn-groupBy-toMap

> Use `Map.groupBy()` and `Object.groupBy()` (ES2024) for grouping data

## Why It Matters

Grouping items by a key is one of the most common data transformations. Before ES2024, it required manual `reduce()` logic that was verbose and easy to get wrong. `Map.groupBy()` returns a `Map` (preserving key types and insertion order), and `Object.groupBy()` returns a plain object. Both are more readable and performant than manual reduction.

## Bad

```js
// Manual reduce — verbose and error-prone
const usersByRole = users.reduce((acc, user) => {
  const key = user.role;
  if (!acc[key]) acc[key] = [];
  acc[key].push(user);
  return acc;
}, {});

// Using a utility library (lodash)
import groupBy from 'lodash/groupBy.js';
const usersByRole = groupBy(users, 'role');
```

## Good

```js
// Map.groupBy() — returns a Map with key of any type
const usersByRole = Map.groupBy(users, user => user.role);

// Access by key
const admins = usersByRole.get('admin') ?? [];

// Iterate over groups
for (const [role, groupUsers] of usersByRole) {
  console.log(`${role}: ${groupUsers.length} users`);
}

// Object.groupBy() — returns a plain object (string keys only)
const usersByRole = Object.groupBy(users, user => user.role);
// { admin: [...], user: [...], guest: [...] }
```

## Common Use Cases

```js
// Group orders by status
const ordersByStatus = Map.groupBy(orders, o => o.status);

// Group by computed key
const byFirstLetter = Map.groupBy(words, w => w[0].toUpperCase());

// Group numbers by parity
const parity = Map.groupBy(numbers, n => n % 2 === 0 ? 'even' : 'odd');

// Group with complex key function
const productsByCategory = Map.groupBy(products, p => p.category?.slug ?? 'uncategorized');
```

## When Exceptions Apply

If you need to support Node.js < 21 or older browsers, use a polyfill or lodash's `groupBy`. `Map.groupBy` is available in Node.js 21+ and modern browsers (Chrome 117+, Firefox 119+, Safari 17.4+).

## See Also

- [fn-map-over-for](./fn-map-over-for.md) - Array method patterns
- [fn-immutability](./fn-immutability.md) - Immutable data patterns
