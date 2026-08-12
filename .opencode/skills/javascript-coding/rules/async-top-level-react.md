# async-top-level-react

> Use top-level await in ES modules (Node.js 14.8+)

## Why It Matters

Top-level await removes the need for an async IIFE wrapper in module entry points. It simplifies startup scripts, configuration loading, and CLI tools. However, it blocks the importing module from executing — use it intentionally in entry points, not in libraries consumed by others.

## Bad

```js
// Async IIFE pattern — unnecessary nesting
import { initDB } from './db.js';

(async () => {
  const db = await initDB();
  await db.migrate();
  startServer(db);
})();
```

## Good

```js
// Top-level await — clean, linear flow
import { initDB } from './db.js';

const db = await initDB();
await db.migrate();
startServer(db);
```

## Caveat: Library Modules

Top-level await blocks consumers of your module. Don't use it in libraries:

```js
// Bad for libraries — blocks anyone who imports this module
export const config = await loadConfig();

// Good for libraries — export a function the caller can await
let _config = null;

export async function getConfig() {
  if (!_config) {
    _config = await loadConfig();
  }
  return _config;
}
```

## When Exceptions Apply

Top-level await is ideal for:
- Application entry points (`server.mjs`, `cli.mjs`)
- Database connection modules that must be ready before the app starts
- One-time initialization scripts
- Test setup files

## See Also

- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - Prefer ES modules over CommonJS
- [mod-dynamic-import](./mod-dynamic-import.md) - Use dynamic import() for lazy loading
