# mod-esm-over-cjs

> Prefer ES modules over CommonJS for new projects

## Why It Matters

ES modules (`import`/`export`) are the standardized module system for JavaScript. They enable static analysis (tree-shaking, dead code elimination), top-level await, and better interop with browsers. CommonJS (`require`/`module.exports`) is legacy and prevents many modern optimizations. Node.js 20+ has mature, stable ESM support.

## Bad

```js
// CommonJS — legacy format
const fs = require('node:fs');
const express = require('express');

module.exports = function handler(req, res) {
  res.send('Hello');
};

// Mixed patterns
import { readFile } from 'node:fs/promises';
const db = require('./database');  // Error in strict ESM
```

## Good

```js
// ES modules — standard format
import { readFile } from 'node:fs/promises';
import express from 'express';

export function handler(req, res) {
  res.send('Hello');
}

// Default export (use sparingly, prefer named)
export default class Server { /* ... */ }
```

## package.json Configuration

```jsonc
{
  "type": "module"  // All .js files are treated as ES modules
}
```

```js
// .mjs — always ESM, regardless of package.json
// .cjs — always CommonJS, regardless of package.json
```

## Migration Path

```js
// Before (CJS)
const { readFile } = require('fs').promises;  // Can't destructure require
// After (ESM)
import { readFile } from 'node:fs/promises';

// Dynamic import for lazy loading
const module = await import('./heavy-module.js');

// Import JSON (ESM)
import config from './config.json' with { type: 'json' };
```

## When Exceptions Apply

CommonJS is still necessary for:
- Projects that depend on many CJS-only packages
- Electron main process (improving but still CJS-dominant)
- Legacy codebases where migration cost is too high
- Some testing frameworks (Jest needs configuration for ESM)

## See Also

- [mod-named-over-default](./mod-named-over-default.md) - Prefer named exports
- [mod-dynamic-import](./mod-dynamic-import.md) - Dynamic import() for lazy loading
- [node-esm-migration](./node-esm-migration.md) - import.meta.url migration patterns
