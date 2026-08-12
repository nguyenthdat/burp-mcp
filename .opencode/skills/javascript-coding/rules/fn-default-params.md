# fn-default-params

> Use default function parameters instead of manual null checks inside the function body

## Why It Matters

Default parameters make the function's contract explicit at the call site. Manual `if (x === undefined)` checks inside the function body are verbose, easy to miss, and separate the default from the parameter declaration. Default parameters are a first-class language feature and should be the primary mechanism for optional arguments.

## Bad

```js
// Manual default checks — verbose, easy to miss
function fetchUsers(page, limit) {
  if (page === undefined) page = 1;
  if (limit === undefined) limit = 10;
  return api.get(`/users?page=${page}&limit=${limit}`);
}

// Using || — bugs with falsy values
function setVolume(level) {
  level = level || 50;  // 0 is a valid volume but becomes 50
  applyVolume(level);
}
```

## Good

```js
// Default parameters — clean and explicit
function fetchUsers(page = 1, limit = 10) {
  return api.get(`/users?page=${page}&limit=${limit}`);
}

// With destructuring
function fetchUsers({ page = 1, limit = 10 } = {}) {
  return api.get(`/users?page=${page}&limit=${limit}`);
}

// Default with nullish coalescing for callers who pass null
function fetchUsers(page = 1, limit = 10) {
  page = page ?? 1;  // Handles explicit null
  limit = limit ?? 10;
}
```

## Computed Defaults

```js
// Default from another parameter
function createRange(start, end = start + 10) {
  return { start, end };
}

// Default from a function call
function generateId(prefix = crypto.randomUUID()) {
  return `${prefix}-${Date.now()}`;
}

// Default from external config
function connect(host = CONFIG.defaultHost, port = CONFIG.defaultPort) {
  return { host, port };
}
```

## When Exceptions Apply

Default parameters are evaluated at call time. If the default is expensive and rarely needed, use a manual check with lazy evaluation:

```js
function process(data, logger = createExpensiveLogger()) {  // Created even if not needed
  // ...
}

function process(data, logger) {
  logger = logger ?? createExpensiveLogger();  // Only created when undefined
}
```

## See Also

- [fn-destructure](./fn-destructure.md) - Destructuring with defaults
- [fn-nullish-coalescing](./fn-nullish-coalescing.md) - ?? for null checks
