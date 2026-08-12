# node-avoid-sync-require

> Don't use `require()` dynamically inside async functions — use dynamic `import()` instead

## Why It Matters

In ES modules, `require()` is not available (unless explicitly created with `createRequire`). Using synchronous `require()` inside async functions blocks the event loop during module loading. Dynamic `import()` is the async-native ES module alternative that works in both CJS and ESM contexts and supports lazy loading without blocking.

## Bad

```js
// require in async function — not available in ESM without createRequire
async function loadModule(name) {
  const module = require(name);  // ReferenceError in ESM
  return module;
}

// Using createRequire as a workaround
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);

async function loadModule(name) {
  const module = require(name);  // Works but blocks the event loop
  return module;
}
```

## Good

```js
// Dynamic import — async, works everywhere
async function loadModule(name) {
  const module = await import(name);
  return module;
}

// Handle both ES modules and CommonJS
async function loadModule(name) {
  try {
    return await import(name);
  } catch (err) {
    if (err.code === 'ERR_REQUIRE_ESM') {
      throw err;  // ES module can't be imported synchronously
    }
    throw err;
  }
}

// Conditional loading
if (condition) {
  const heavy = await import('./heavy-module.js');
  heavy.process();
}
```

## When Exceptions Apply

`require()` is still standard in CommonJS files. This rule applies to async functions in ES modules. In CJS, `require()` at the top of the file is idiomatic.

## See Also

- [mod-dynamic-import](./mod-dynamic-import.md) - Dynamic import() for lazy loading
- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ES modules preference
