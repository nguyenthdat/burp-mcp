# lint-consistent-type-imports

> Enforce `consistent-type-imports` so type-only imports are marked explicitly

## Why It Matters

An import used only for its type (`import { Config } from './config'` where `Config` is an interface) looks identical, at the import statement, to an import of a runtime value — the reader (and any single-file transpiler) can't tell which it is without checking the source module. If that module is later refactored to remove the exported value entirely, a type-only usage keeps compiling fine under `tsc`, but a build using esbuild/SWC that doesn't do full type-checking may either keep a now-dead import (harmless but wasteful) or, in edge cases, produce different tree-shaking results than intended. `@typescript-eslint/consistent-type-imports` auto-fixes every import to explicitly mark type-only specifiers with `type`, making intent visible and machine-verifiable rather than inferred.

## Bad

```typescript
import { Config, loadConfig } from './config'; // Config is a type, loadConfig is a value — ambiguous at a glance

function apply(config: Config) {
  return loadConfig(config);
}
```

## Good

```javascript
// eslint.config.js
export default tseslint.config({
  rules: {
    '@typescript-eslint/consistent-type-imports': [
      'error',
      { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
    ],
  },
});
```

```typescript
import { type Config, loadConfig } from './config'; // explicit and auto-fixed

function apply(config: Config) {
  return loadConfig(config);
}
```

## `fixStyle` Options

| `fixStyle` | Output |
|---|---|
| `separate-type-imports` | `import type { Config } from './config';` on its own line |
| `inline-type-imports` | `import { type Config, loadConfig } from './config';` combined |

`inline-type-imports` pairs well with `verbatimModuleSyntax`, since both aim for the same explicitness without forcing every type import into a separate statement. Run `eslint --fix` to apply this automatically across an existing codebase in one pass.

## See Also

- [proj-verbatim-module-syntax](proj-verbatim-module-syntax.md) - Enable `verbatimModuleSyntax` for unambiguous type-only imports/exports
- [proj-isolated-modules](proj-isolated-modules.md) - Enable `isolatedModules` for compatibility with single-file transpilers
- [lint-no-unused-vars](lint-no-unused-vars.md) - Enable the TypeScript-aware `no-unused-vars` rule
