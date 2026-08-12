# proj-verbatim-module-syntax

> Enable `verbatimModuleSyntax` for unambiguous type-only imports/exports

## Why It Matters

TypeScript can erase type-only imports at compile time, but before `verbatimModuleSyntax` existed, the rules for *when* an import got erased were implicit and version-dependent (`isolatedModules`, `importsNotUsedAsValues`, and `preserveValueImports` interacted in confusing ways). That ambiguity caused real bugs: an import that TypeScript decided was "type-only" and erased could turn out to be needed at runtime for its side effects, or a single-file transpiler (esbuild, SWC, Babel) — which can't do full type analysis — would guess wrong and either keep an import that should have been erased or drop one that shouldn't have been. `verbatimModuleSyntax` removes the ambiguity: every import/export is emitted exactly as-written unless explicitly marked `type`, and any import that would be erroneously elided must be explicitly annotated with `import type` or `export type`.

## Bad

```typescript
// Without verbatimModuleSyntax, it's unclear to a reader (and to esbuild/SWC)
// whether this import survives compilation or gets erased as type-only.
import { Config } from './config';
import { validate } from './validate';

export function loadConfig(): Config {
  return validate(rawConfig);
}
```

## Good

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "verbatimModuleSyntax": true
  }
}
```

```typescript
// Explicit: Config is type-only and will be erased; validate is a value and stays.
import type { Config } from './config';
import { validate } from './validate';

export function loadConfig(): Config {
  return validate(rawConfig);
}
```

```typescript
// Mixed imports use inline `type` markers on individual specifiers
import { type Config, DEFAULT_CONFIG } from './config';
```

## What Changes When You Enable It

| Before | After `verbatimModuleSyntax` |
|---|---|
| TS silently elides imports it infers are type-only | You must write `import type` / inline `type` explicitly, or the import is kept and may error if the target has no runtime export |
| CommonJS `import foo = require(...)` sometimes worked in ESM output | Disallowed in ESM contexts; must use `import foo from ...` |
| Ambiguous behavior across transpilers (esbuild vs `tsc`) | Every transpiler behaves identically because there's no type inference required |

Combine with `lint-consistent-type-imports` (the ESLint rule) so violations are caught before `tsc` even runs.

## See Also

- [lint-consistent-type-imports](lint-consistent-type-imports.md) - Enforce `consistent-type-imports` so type-only imports are marked explicitly
- [proj-isolated-modules](proj-isolated-modules.md) - Enable `isolatedModules` for compatibility with single-file transpilers
- [node-esm-first](node-esm-first.md) - Prefer ES modules over CommonJS for new Node.js projects
