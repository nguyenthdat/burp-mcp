# node-package-exports-map

> Define package entry points with the `exports` field

## Why It Matters

Without an `exports` map, every file in a package is importable, so consumers reach into internal implementation files (`my-lib/dist/internal/helpers.js`) that were never meant to be public API. That turns internal refactors into breaking changes. The `exports` field is enforced by Node.js's resolver (not just a convention): it restricts which subpaths are importable, lets you ship different entry points for `import` vs `require`, and lets you provide conditional exports for `types`, `browser`, or `development`/`production` builds. Relying only on `main`/`module`/`types` fields is also ambiguous about which build a bundler should pick, causing dual-package hazards where two copies of the same module end up loaded.

## Bad

```jsonc
// package.json - only "main", no boundary enforcement
{
  "name": "my-lib",
  "main": "dist/index.js",
  "types": "dist/index.d.ts"
}
```

```typescript
// Consumers can reach into internals; nothing stops this
import { formatDate } from 'my-lib/dist/internal/date-utils.js';
```

## Good

```jsonc
// package.json
{
  "name": "my-lib",
  "type": "module",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js",
      "require": "./dist/index.cjs"
    },
    "./client": {
      "types": "./dist/client.d.ts",
      "import": "./dist/client.js"
    },
    "./package.json": "./package.json"
  },
  "main": "./dist/index.cjs",
  "module": "./dist/index.js",
  "types": "./dist/index.d.ts"
}
```

```typescript
// Only the declared subpaths are importable
import { formatDate } from 'my-lib';
import { createClient } from 'my-lib/client';

// This now throws ERR_PACKAGE_PATH_NOT_EXPORTED at resolution time
// import x from 'my-lib/dist/internal/date-utils.js';
```

## Condition Order Matters

The `exports` object's keys are matched top-to-bottom for the first match, so put more specific conditions (`types`) before generic ones (`import`/`require`). Most tools require `types` to come first in each conditional block or type resolution silently falls back to `any`.

| Condition | Consumer |
|---|---|
| `types` | TypeScript's module resolution |
| `import` | ESM `import` statements |
| `require` | CommonJS `require()` |
| `browser` | Bundlers targeting browsers (webpack, Vite) |
| `default` | Fallback, must be last |

## See Also

- [node-esm-first](node-esm-first.md) - Prefer ES modules over CommonJS for new Node.js projects
- [proj-declaration-files](proj-declaration-files.md) - Emit `.d.ts` declaration files for any published package
- [api-module-boundary-types](api-module-boundary-types.md) - Keep module boundary types explicit
- [proj-module-boundaries](proj-module-boundaries.md) - Enforce module boundaries; don't import another module's internal files
