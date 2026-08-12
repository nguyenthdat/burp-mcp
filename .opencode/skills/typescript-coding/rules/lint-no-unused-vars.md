# lint-no-unused-vars

> Enable the TypeScript-aware `no-unused-vars` rule

## Why It Matters

ESLint's base `no-unused-vars` rule doesn't understand TypeScript-only constructs — it will flag unused type parameters, interface members, or imports used only in type positions as errors, or conversely miss unused ones entirely, because it wasn't built with type-level syntax in mind. `@typescript-eslint/no-unused-vars` replaces it with a version that correctly understands types, generics, and declaration merging, catching genuinely dead code (an imported helper nobody calls, a destructured variable nobody reads) which otherwise accumulates silently, bloats bundles marginally, and confuses readers about what's actually in use.

## Bad

```javascript
// eslint.config.js — base ESLint rule, mishandles TS-specific syntax
export default [
  {
    rules: {
      'no-unused-vars': 'error', // wrong rule for a TS codebase
    },
  },
];
```

```typescript
import { formatDate, parseDate } from './date-utils'; // parseDate never used
import type { Config } from './config'; // flagged incorrectly by the base rule

function render(items: Item[]) {
  const [first, ...rest] = items; // `rest` unused, silently ignored
  return formatDate(first.date);
}
```

## Good

```javascript
// eslint.config.js
import tseslint from 'typescript-eslint';

export default tseslint.config({
  rules: {
    'no-unused-vars': 'off', // turn off the base rule entirely
    '@typescript-eslint/no-unused-vars': [
      'error',
      {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_',
      },
    ],
  },
});
```

```typescript
import { formatDate } from './date-utils'; // unused parseDate now flagged and removed

function render(items: Item[]) {
  const [first, ..._rest] = items; // intentional discard, prefixed with _
  return formatDate(first.date);
}

function handler(_req: Request, res: Response) {
  // `_req` intentionally unused — matches argsIgnorePattern, no warning
  res.send('ok');
}
```

## See Also

- [lint-consistent-type-imports](lint-consistent-type-imports.md) - Enforce `consistent-type-imports` so type-only imports are marked explicitly
- [lint-typescript-eslint-recommended](lint-typescript-eslint-recommended.md) - Adopt `typescript-eslint`'s recommended (or recommended-type-checked) config
- [perf-tree-shaking-friendly](perf-tree-shaking-friendly.md) - Write modules so bundlers can tree-shake unused exports
