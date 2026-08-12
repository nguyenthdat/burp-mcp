# fn-destructure

> Use destructuring for function parameters and object/array access

## Why It Matters

Destructuring extracts values from objects and arrays into named variables in a single expression. For function parameters, it makes the expected shape of the input explicit, provides natural defaults, and replaces repetitive `obj.prop` access patterns. It reduces line count and cognitive load.

## Bad

```js
// Repetitive property access
function createUser(data) {
  const name = data.name;
  const email = data.email;
  const role = data.role || 'user';
  const active = data.active !== undefined ? data.active : true;
  return { name, email, role, active };
}

// Positional parameters — order matters, hard to read at call site
function createUser(name, email, role, active) {
  return { name, email, role, active };
}
createUser('Alice', 'alice@example.com', 'admin', true);
// What does true mean? Which position is role?
```

## Good

```js
// Object destructuring with defaults
function createUser({ name, email, role = 'user', active = true }) {
  return { name, email, role, active };
}

createUser({
  name: 'Alice',
  email: 'alice@example.com',
  role: 'admin',
});

// Array destructuring
const [first, second, ...rest] = items;
const [x = 0, y = 0] = point;

// Nested destructuring
const {
  address: { city, country },
} = user;

// Renaming during destructure
const { name: userName, email: userEmail } = data;

// Destructuring in loops
for (const { id, name } of users) {
  console.log(`${id}: ${name}`);
}
```

## Destructured Options Pattern

```js
function fetchData(url, { method = 'GET', headers = {}, timeout = 5000 } = {}) {
  // url is a positional arg, options is a destructured object
}

fetchData('/api/users', { timeout: 10000 });
fetchData('/api/users');  // Options default to {}
```

## When Exceptions Apply

For functions with 1-2 simple parameters, destructuring adds unnecessary syntax. Use positional parameters for obvious, mandatory arguments.

## See Also

- [fn-default-params](./fn-default-params.md) - Default parameter values
- [fn-rest-spread](./fn-rest-spread.md) - Rest/spread operators
