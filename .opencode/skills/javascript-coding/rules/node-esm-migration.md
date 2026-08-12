# node-esm-migration

> Use `import.meta.url` instead of `__dirname` and `__filename` in ES modules

## Why It Matters

`__dirname` and `__filename` are CommonJS-only globals and do not exist in ES modules. Using them in `.mjs` files or packages with `"type": "module"` causes a ReferenceError. `import.meta.url` provides the module's file URL, and `import.meta.dirname` / `import.meta.filename` (Node.js 21.2+) provide the path directly.

## Bad

```js
// __dirname and __filename don't exist in ESM
const configPath = __dirname + '/config.json';  // ReferenceError!
const currentFile = __filename;                  // ReferenceError!

// Manual URL→path conversion
import { fileURLToPath } from 'node:url';
import { dirname } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
```

## Good

```js
// Node.js 21.2+ — built-in
const configPath = new URL('./config.json', import.meta.url);
const currentDir = import.meta.dirname;
const currentFile = import.meta.filename;

// Pre-21.2 compatibility
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const configPath = resolve(__dirname, 'config.json');
```

## Common Patterns

```js
// Read a file relative to the current module
import { readFile } from 'node:fs/promises';

const packageJson = JSON.parse(
  await readFile(new URL('./package.json', import.meta.url), 'utf8'),
);

// Resolve a path relative to the module
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const moduleDir = fileURLToPath(new URL('.', import.meta.url));
const templatePath = resolve(moduleDir, 'templates/email.html');
```

## When Exceptions Apply

If supporting Node.js < 21.2, you need the `fileURLToPath` + `dirname` workaround. In CommonJS files, `__dirname` and `__filename` remain the standard.

## See Also

- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ES modules over CommonJS
- [node-path-join](./node-path-join.md) - Path construction
