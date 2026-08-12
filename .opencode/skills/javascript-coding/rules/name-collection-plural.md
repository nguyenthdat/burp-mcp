# name-collection-plural

> Use plural names for arrays, Sets, Maps, and collections

## Why It Matters

Plural names immediately signal that a variable holds multiple items, making iteration and access patterns obvious. A singular name for an array is misleading — `user` suggests a single object, while `users` clearly indicates a collection. This convention prevents bugs where developers accidentally treat a collection as a single item.

## Bad

```js
// Singular names for collections — misleading
const user = await fetchUsers();  // Is this one user or many?
const result = items.map(i => i.name);  // result suggests a single value
const orderList = getOrders();  // Redundant — 'List' adds noise
const data = [1, 2, 3];  // Too generic
```

## Good

```js
// Plural names — immediately clear
const users = await fetchUsers();
const names = items.map(item => item.name);
const orders = getOrders();
const numbers = [1, 2, 3];

// Specific collective names
const userMap = new Map();    // Map of users
const userIdSet = new Set();  // Set of IDs
const userById = {};          // Object used as a lookup (descriptive key)

// Iterating reads naturally
for (const user of users) { /* ... */ }
users.forEach(user => { /* ... */ });
```

## Collection Type Suffixes

```js
// For clarity when multiple collections exist
const userIds = users.map(u => u.id);           // Array of IDs
const userMap = new Map(userIds.map(id => [id, getUser(id)]));  // Map
const activeUserSet = new Set(activeUsers);     // Set

// Lookup objects
const usersByEmail = {};
const ordersByStatus = {};
```

## When Exceptions Apply

Some domains prefer singular collection names (e.g., `fruit` for a list of fruits in a grocery app). Follow your domain's conventions, but prefer plural in general-purpose code.

## See Also

- [name-camelCase](./name-camelCase.md) - camelCase convention
- [name-avoid-single-letter](./name-avoid-single-letter.md) - Avoid single-letter names
