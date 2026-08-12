# proj-declaration-files

> Emit `.d.ts` declaration files for any published package

## Why It Matters

A published package without `.d.ts` files forces every TypeScript consumer to either write their own ambient declarations, fall back to `any` for the whole module, or pull in a possibly-outdated `@types/*` package maintained by someone else entirely. Shipping your own `.d.ts` files keeps the types authored alongside (and guaranteed consistent with) the implementation, gives consumers autocomplete and inline documentation, and is a basic expectation for any TypeScript-first package in the current ecosystem — the compiler generates them for you from your source, so there's no separate type-definition file to maintain by hand.

## Bad

```jsonc
// package.json — no "types" field, no .d.ts emitted at all
{
  "name": "my-lib",
  "main": "dist/index.js"
}
```

```typescript
// Consumer's project
import { createClient } from 'my-lib'; // typed as `any`, no autocomplete, no safety
```

## Good

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "declaration": true,
    "declarationMap": true, // lets consumers "go to definition" into your .ts source
    "outDir": "./dist"
  }
}
```

```jsonc
// package.json
{
  "name": "my-lib",
  "type": "module",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  },
  "types": "./dist/index.d.ts",
  "files": ["dist"]
}
```

```bash
tsc --build   # emits dist/index.js, dist/index.d.ts, dist/index.d.ts.map
```

## Bundling Declarations for Dual CJS/ESM Packages

If you publish both a CJS and ESM build, use a build tool like `tsup` to bundle declaration files per entry point rather than hand-rolling separate `.d.ts`/`.d.cts` files:

```jsonc
// tsup.config.ts
{
  "entry": ["src/index.ts"],
  "format": ["esm", "cjs"],
  "dts": true, // emits index.d.ts and index.d.cts automatically
  "clean": true
}
```

Run `tsc --noEmit` in CI on a throwaway consumer snippet (or use `@arethetypeswrong/cli`) to catch cases where your `exports` map and declaration files disagree — a common source of "works for me, broken for consumers" bug reports.

## See Also

- [node-package-exports-map](node-package-exports-map.md) - Define package entry points with the `exports` field
- [node-esm-first](node-esm-first.md) - Prefer ES modules over CommonJS for new Node.js projects
- [doc-tsdoc-public-api](doc-tsdoc-public-api.md) - Document public API surfaces with TSDoc
