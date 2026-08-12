# node-esm-first

> Prefer ES modules over CommonJS for new Node.js projects

## Why It Matters

CommonJS's synchronous `require()` cannot load ESM-only packages, blocks top-level `await`, and forces workarounds like dynamic `import()` just to consume modern dependencies. ESM is the standard JavaScript module system, supported natively by Node.js, browsers, and bundlers, so code written as ESM is portable without a transpilation step. Sticking with CommonJS in new projects means fighting the ecosystem: an increasing share of packages (e.g. `chalk`, `execa`, `nanoid`) publish ESM-only, and CJS consumers must resort to fragile dynamic imports to use them. ESM also enables static analysis (tree-shaking, better circular-import detection) that CommonJS's dynamic `require` graph can't support.

## Bad

```typescript
// package.json has no "type" field, so this is CommonJS
// utils.js
const fs = require('fs');
const path = require('path');

function readConfig(name) {
  return JSON.parse(fs.readFileSync(path.join(__dirname, name), 'utf8'));
}

module.exports = { readConfig };

// index.js
const { readConfig } = require('./utils');
// Cannot use top-level await; cannot cleanly import ESM-only deps
readConfig('config.json');
```

## Good

```typescript
// package.json: { "type": "module" }
// utils.ts
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export function readConfig(name: string) {
  return JSON.parse(fs.readFileSync(path.join(__dirname, name), 'utf8'));
}

// index.ts
import { readConfig } from './utils.js'; // note the .js extension in ESM
import { nanoid } from 'nanoid'; // ESM-only package works natively

const id = nanoid();
const config = await readConfig('config.json'); // top-level await
```

## Migration Notes

- Set `"type": "module"` in `package.json`, and `"module": "NodeNext"` / `"moduleResolution": "NodeNext"` in `tsconfig.json`.
- Relative import specifiers must include the emitted file extension (`./utils.js`, not `./utils`), even when the source file is `utils.ts`.
- `__dirname` and `__filename` don't exist in ESM; derive them from `import.meta.url` when needed.
- If you must publish for both CJS and ESM consumers, use dual-package builds (see `proj-declaration-files`) rather than staying CJS-only.
- Jest historically had rough ESM support; Vitest works with ESM out of the box and is a common reason teams switch test runners alongside this migration.

## See Also

- [node-package-exports-map](node-package-exports-map.md) - Define package entry points with the `exports` field
- [proj-verbatim-module-syntax](proj-verbatim-module-syntax.md) - Enable `verbatimModuleSyntax` for unambiguous type-only imports/exports
- [async-top-level-await](async-top-level-await.md) - Use top-level await where the module system supports it
- [proj-declaration-files](proj-declaration-files.md) - Emit `.d.ts` declaration files for any published package
