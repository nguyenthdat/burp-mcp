# doc-type-annotations

> Use `@type`, `@typedef`, and `@property` in JSDoc to document types in plain JavaScript

## Why It Matters

Without type annotations, plain JavaScript provides no information about variable shapes, making large codebases hard to navigate. JSDoc type annotations enable IDE autocompletion, inline documentation, and TypeScript-style type checking with `--checkJs`. They're the bridge between untyped JavaScript and type-safe development.

## Bad

```js
// No type hints — user must guess the shape
const config = {
  port: 3000,
  database: {
    host: 'localhost',
    port: 5432,
    name: 'myapp',
  },
};

function createUser(data) {
  // What properties does `data` have?
  // What does this return?
}

let userCache;  // What type is this?
```

## Good

```js
/**
 * @typedef {Object} DatabaseConfig
 * @property {string} host
 * @property {number} port
 * @property {string} name
 * @property {string} [password]
 */

/**
 * @typedef {Object} AppConfig
 * @property {number} port
 * @property {DatabaseConfig} database
 */

/** @type {AppConfig} */
const config = {
  port: 3000,
  database: {
    host: 'localhost',
    port: 5432,
    name: 'myapp',
  },
};

/**
 * @param {UserInput} data
 * @returns {Promise<User>}
 */
function createUser(data) { /* ... */ }

/** @type {Map<string, User>} */
let userCache;

// IDE now provides autocompletion for config.database.host
```

## Common Type Annotations

```js
/** @type {string} */
const name = 'Alice';

/** @type {number[]} */
const scores = [95, 87, 92];

/** @type {Map<string, User>} */
const users = new Map();

/** @type {Set<number>} */
const ids = new Set();

/** @type {Promise<User[]>} */
const userPromise = fetchUsers();

/** @type {(a: number, b: number) => number} */
const add = (a, b) => a + b;

/** @type {{ name: string, age: number }} */
const person = { name: 'Bob', age: 30 };
```

## TypeScript-Like Checking

```bash
# Add to package.json scripts
{
  "typecheck": "tsc --noEmit --allowJs --checkJs"
}

# Or use in CI
npx -y typescript --noEmit --allowJs --checkJs
```

## When Exceptions Apply

Type annotations are most valuable for public APIs, complex object shapes, and module boundaries. Internal variables with obvious types (`const name = 'Alice'`) don't need them.

## See Also

- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
- [doc-param-return](./doc-param-return.md) - Parameter documentation
