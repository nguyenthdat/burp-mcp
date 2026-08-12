# type-null-over-undefined

> Prefer `null` for intentional absence of a value; reserve `undefined` for "not yet assigned"

## Why It Matters

JavaScript has two bottom values: `null` (intentional absence) and `undefined` (not assigned). Using both arbitrarily creates confusion about whether a value was intentionally set to "nothing" or just missing. Convention helps: `null` means "I deliberately set this to empty"; `undefined` means "this hasn't been set yet."

## Bad

```js
// Using undefined to mean "no value set"
function getUser(id) {
  return database.find(id);  // Returns undefined if not found
}

// Mixing null and undefined inconsistently
function process(data) {
  if (data.result === undefined) return;
  if (data.error === null) return;
}

// Using both in the same API response
res.json({ user: undefined, count: null });  // user key is omitted, count is null
```

## Good

```js
// null for intentional absence
function getUser(id) {
  const user = database.find(id);
  return user ?? null;  // Normalize: return null if not found
}

// Consistent with nullish coalescing
const name = user?.name ?? null;

// API responses: use null, not undefined
res.json({ user: null, count: 0 });  // Both properties present

// undefined for optional parameters
function createUser(name, email, role) {
  // role is undefined if not provided — natural default
}
```

## Convention

```js
// Database queries: return null for not found
async function findById(id) {
  const rows = await db.query('SELECT * FROM users WHERE id = ?', [id]);
  return rows[0] ?? null;
}

// JSON serialization: null is preserved, undefined is dropped
JSON.stringify({ a: null, b: undefined });  // '{"a":null}'

// Default parameters: undefined triggers default, null doesn't
function greet(name = 'stranger') {
  return `Hello, ${name}`;
}
greet(undefined);  // "Hello, stranger"
greet(null);       // "Hello, null"
```

## When Exceptions Apply

In some frameworks and libraries, `undefined` is the convention. Follow the framework's standard. When in doubt, prefer `null` for intentional absence in your own code.

## See Also

- [fn-nullish-coalescing](./fn-nullish-coalescing.md) - ?? for null/undefined checks
- [fn-optional-chaining](./fn-optional-chaining.md) - ?. for safe access
