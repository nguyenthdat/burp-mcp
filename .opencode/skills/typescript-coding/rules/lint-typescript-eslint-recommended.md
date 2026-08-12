# lint-typescript-eslint-recommended

> Adopt `typescript-eslint`'s recommended (or recommended-type-checked) config

## Why It Matters

Hand-picking individual ESLint rules for a TypeScript codebase means you inevitably miss ones that catch real bugs (floating promises, unsafe `any` propagation, unnecessary type assertions) simply because nobody thought to enable them. `typescript-eslint`'s recommended configs are curated and maintained by the same team that maintains the TypeScript ESLint parser, updated as TypeScript itself evolves, and split into tiers so you can choose the depth of checking versus the performance cost (type-checked rules require a full type-checker pass and are slower on large codebases, but catch categorically more bugs).

## Bad

```javascript
// eslint.config.js — reinventing the wheel with an ad hoc rule list,
// missing dozens of rules the maintainers already know matter
export default [
  {
    rules: {
      'no-unused-vars': 'warn',
      eqeqeq: 'error',
    },
  },
];
```

## Good

```javascript
// eslint.config.js (flat config, ESLint 9+)
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
);
```

```bash
npm install --save-dev eslint typescript-eslint
```

## Config Tiers

| Config | Type info required | Use for |
|---|---|---|
| `recommended` | No | Fast, syntax-only checks; good baseline for any project |
| `recommendedTypeChecked` | Yes | Catches bugs that need type information (floating promises, unsafe `any` flow) |
| `strict` / `strictTypeChecked` | Varies | Additional rules that are correct but more opinionated; adopt incrementally |
| `stylistic` / `stylisticTypeChecked` | Varies | Formatting-adjacent rules not covered by Prettier |

Start with `recommendedTypeChecked` for any serious project; the type-checked pass is the tier that catches `async` functions passed where a sync callback is expected, unhandled promise rejections, and other bugs a syntax-only linter structurally cannot see.

## See Also

- [lint-no-floating-promises-rule](lint-no-floating-promises-rule.md) - Enable `@typescript-eslint/no-floating-promises`
- [lint-strict-tsconfig](lint-strict-tsconfig.md) - Enable `strict: true` and other strictness flags in `tsconfig.json`
- [lint-ci-lint-gate](lint-ci-lint-gate.md) - Run typecheck and lint as a required CI gate
- [lint-prettier-integration](lint-prettier-integration.md) - Use Prettier for formatting and let ESLint own only code-quality rules
