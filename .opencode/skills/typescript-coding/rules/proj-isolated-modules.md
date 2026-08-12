# proj-isolated-modules

> Enable `isolatedModules` for compatibility with single-file transpilers

## Why It Matters

Tools like esbuild, SWC, and Babel transpile TypeScript to JavaScript one file at a time, with no knowledge of any other file in the project — they strip types without ever running full type-checking. That's what makes them fast, but it means they can't see across file boundaries to determine, for example, whether a re-exported symbol is a type or a value, or resolve `const enum` values that live in another file. A handful of valid `tsc`-only TypeScript patterns are ambiguous or impossible for a single-file transpiler to compile correctly. `isolatedModules` makes `tsc` itself flag any code that relies on cross-file type information, so you find out at typecheck time — not when your bundler silently miscompiles a file in production.

## Bad

```jsonc
// tsconfig.json — isolatedModules not enabled, so tsc won't warn about
// patterns that a single-file transpiler cannot safely compile
{
  "compilerOptions": {
    "target": "ES2022"
  }
}
```

```typescript
// const enum requires cross-file inlining knowledge; esbuild/SWC can't do this safely
export const enum Direction {
  Up,
  Down,
}

// Re-exporting a type without `export type` is ambiguous to a
// single-file transpiler: is Config a type or a value?
export { Config } from './types';
```

## Good

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "isolatedModules": true,
    "verbatimModuleSyntax": true
  }
}
```

```typescript
// tsc now errors: "the 'const' modifier can only be used in TypeScript files
// that are compiled by tsc" style diagnostics steer you away from const enum.
export enum Direction {
  Up,
  Down,
}

// Explicit — safe for any single-file transpiler
export type { Config } from './types';
```

## What `isolatedModules` Catches

| Pattern | Why it's unsafe for single-file transpilation |
|---|---|
| `const enum` | Requires inlining values from the declaration site, which needs cross-file analysis |
| Ambiguous re-exports of types (`export { Foo }` where `Foo` is a type) | Transpiler can't tell if `Foo` is a type (erase) or a value (keep) without checking the source module |
| Non-module files (scripts without any `import`/`export`) | Each file must be independently a module for isolated compilation to make sense |

This flag is required (and usually auto-enabled) by Vite, Next.js, and any esbuild/SWC-based build pipeline — if you use one of those, you likely already need this on.

## See Also

- [proj-verbatim-module-syntax](proj-verbatim-module-syntax.md) - Enable `verbatimModuleSyntax` for unambiguous type-only imports/exports
- [lint-consistent-type-imports](lint-consistent-type-imports.md) - Enforce `consistent-type-imports` so type-only imports are marked explicitly
- [perf-bundle-size-audit](perf-bundle-size-audit.md) - Audit bundle size regularly
