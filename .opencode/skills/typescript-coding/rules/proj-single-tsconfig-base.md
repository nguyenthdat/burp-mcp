# proj-single-tsconfig-base

> Share a base `tsconfig.json` and extend it per package

## Why It Matters

When every package in a monorepo maintains its own independent `tsconfig.json`, compiler strictness settings drift: one package enables `strict`, another forgets `noUncheckedIndexedAccess`, and a third targets an outdated `lib`. That inconsistency means the same bug class is caught in one package and silently allowed in another, and upgrading TypeScript-wide settings requires editing N files instead of one. A shared base config with `extends` centralizes the strictness and compilation-target decisions in one place, while still letting each package override the handful of settings that legitimately differ (its `outDir`, whether it emits declarations, its `rootDir`).

## Bad

```jsonc
// packages/api/tsconfig.json
{ "compilerOptions": { "strict": true, "target": "ES2022", "module": "NodeNext" } }

// packages/worker/tsconfig.json — drifted, missing strict
{ "compilerOptions": { "target": "ES2020", "module": "commonjs" } }

// packages/ui/tsconfig.json — drifted again, different target
{ "compilerOptions": { "strict": true, "target": "ES2021", "jsx": "react-jsx" } }
```

## Good

```jsonc
// tsconfig.base.json (repo root)
{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "noUncheckedIndexedAccess": true,
    "verbatimModuleSyntax": true
  }
}
```

```jsonc
// packages/api/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src"]
}
```

```jsonc
// packages/ui/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "jsx": "react-jsx",
    "outDir": "./dist",
    "rootDir": "./src"
  },
  "include": ["src"]
}
```

## TypeScript Project References

For build performance in large monorepos, combine the shared base with project references so `tsc --build` only recompiles packages whose dependencies changed:

```jsonc
// packages/api/tsconfig.json
{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": { "composite": true, "outDir": "./dist" },
  "references": [{ "path": "../shared" }]
}
```

## See Also

- [proj-monorepo-workspaces](proj-monorepo-workspaces.md) - Use workspaces (pnpm/npm/yarn) to manage a monorepo's packages
- [lint-strict-tsconfig](lint-strict-tsconfig.md) - Enable `strict: true` and other strictness flags in `tsconfig.json`
- [lint-no-unchecked-indexed-access](lint-no-unchecked-indexed-access.md) - Enable `noUncheckedIndexedAccess` in `tsconfig.json`
