# mod-package-exports

> Use the `exports` field in package.json to define the public API surface

## Why It Matters

The `exports` field (also called "export map") explicitly defines which files consumers can import, preventing internal modules from being accessed. It supports conditional exports for ESM/CJS dual packaging, subpath exports for organized imports, and encapsulation of internal implementation details.

## Bad

```jsonc
// No exports field — consumers can import any file
{
  "name": "my-library",
  "main": "./dist/index.cjs",
  "module": "./dist/index.mjs"
}

// Consumer can access internal modules
import { internalHelper } from 'my-library/dist/internal/helper.js';
```

## Good

```jsonc
// Explicit exports — controlled public API
{
  "name": "my-library",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs"
    },
    "./utils": {
      "import": "./dist/utils.mjs",
      "require": "./dist/utils.cjs"
    },
    "./package.json": "./package.json"
  }
}
```

## Subpath Exports for Organized APIs

```jsonc
{
  "name": "my-orm",
  "exports": {
    ".": "./dist/index.js",
    "./postgres": "./dist/drivers/postgres.js",
    "./mysql": "./dist/drivers/mysql.js",
    "./sqlite": "./dist/drivers/sqlite.js",
    "./types": "./dist/types.js"
  }
}

// Consumer usage
import { createConnection } from 'my-orm';
import { createConnection } from 'my-orm/postgres';
import { Column, Table } from 'my-orm/types';
```

## Encapsulation with Exports

```js
// With exports field, this import fails:
import { InternalHelper } from 'my-library/dist/internal/helper.js';
// Error: Package subpath './dist/internal/helper.js' is not defined by "exports"

// Only the defined exports are accessible:
import { publicAPI } from 'my-library';
import { utilities } from 'my-library/utils';
```

## When Exceptions Apply

The `exports` field is required for libraries published to npm. For application code (not published), it's optional but still useful for monorepo workspaces.

## See Also

- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ES module preferences
- [proj-monorepo-over-mono](./proj-monorepo-over-mono.md) - Workspace configuration
