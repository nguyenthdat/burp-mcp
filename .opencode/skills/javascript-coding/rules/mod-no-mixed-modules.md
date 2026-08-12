# mod-no-mixed-modules

> Don't mix `import` and `require` in the same file

## Why It Matters

Mixing ESM `import` statements with CJS `require()` calls in the same file creates confusion about which module system is in use, causes subtle interop bugs, and makes static analysis tools unreliable. Pick one module system per file and be consistent.

## Bad

```js
// Mixing import and require — confusing and error-prone
import express from 'express';
const helmet = require('helmet');  // Mixing CJS in ESM file

export const app = express();
app.use(helmet());

// Conditional require inside ESM
if (process.env.NODE_ENV === 'development') {
  const devTools = require('./dev-tools');  // Mixing again
}
```

## Good

```js
// Pure ESM — consistent
import express from 'express';
import helmet from 'helmet';

export const app = express();
app.use(helmet());

// Dynamic import for conditional loading in ESM
if (process.env.NODE_ENV === 'development') {
  const { default: devTools } = await import('./dev-tools.js');
}
```

## CJS File Pattern

```js
// Pure CJS — consistent
const express = require('express');
const helmet = require('helmet');

module.exports = { app: express() };
```

## When Exceptions Apply

In dual-package configurations (supporting both ESM and CJS consumers), you may have conditional exports in `package.json`. But each individual file should still use one module system:

```jsonc
{
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs"
    }
  }
}
```

## See Also

- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - Prefer ES modules
- [mod-dynamic-import](./mod-dynamic-import.md) - Use dynamic import() for lazy loading
