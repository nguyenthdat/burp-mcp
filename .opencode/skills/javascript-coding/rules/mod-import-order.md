# mod-import-order

> Organize imports in a consistent order: Node builtins → npm packages → local modules

## Why It Matters

A consistent import order makes it easy to scan dependencies, spot missing or duplicate imports, and understand a module's external dependencies at a glance. It also prevents merge conflicts from import shuffling and helps linters enforce consistency.

## Bad

```js
// Chaotic import order
import { User } from './models/user.js';
import fs from 'node:fs';
import express from 'express';
import { config } from './config.js';
import path from 'node:path';
import helmet from 'helmet';
import { logger } from './utils/logger.js';
```

## Good

```js
// 1. Node.js built-ins
import fs from 'node:fs';
import path from 'node:path';

// 2. npm packages
import express from 'express';
import helmet from 'helmet';

// 3. Local modules
import { config } from './config.js';
import { logger } from './utils/logger.js';
import { User } from './models/user.js';
```

## ESLint Auto-Fix

```js
// eslint.config.mjs
import importPlugin from 'eslint-plugin-import';

export default [
  {
    plugins: { import: importPlugin },
    rules: {
      'import/order': ['error', {
        'groups': ['builtin', 'external', 'internal', 'parent', 'sibling', 'index'],
        'newlines-between': 'always',
        'alphabetize': { order: 'asc', caseInsensitive: true },
      }],
    },
  },
];
```

## Grouping Convention

```js
// With blank lines between groups
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import express from 'express';
import pino from 'pino';

import { UserService } from '#services/user-service.js';
import { database } from './database.js';
import { API_URL } from './constants.js';
```

## When Exceptions Apply

In files that mix side-effect imports (e.g., `import './setup.js'`), place those at the top after builtins. CSS/font imports in bundler contexts go at the bottom.

## See Also

- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ES module syntax
- [lint-eslint-setup](./lint-eslint-setup.md) - ESLint flat config
