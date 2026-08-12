# lint-prettier-integration

> Use Prettier for formatting and let ESLint own only code-quality rules

## Why It Matters

ESLint's stylistic rules (`indent`, `quotes`, `max-len`, etc.) and Prettier solve the same problem — consistent formatting — but Prettier is a dedicated formatter with an opinionated, near-zero-config algorithm, while ESLint's formatting rules are configurable in ways that invite bikeshedding and can conflict with each other or with Prettier's own output. Running both unconfigured means ESLint and Prettier fight over the same lines, producing a save-and-fix loop that never converges, or CI failures caused purely by formatting disagreements rather than real bugs. The current best practice is a clean split: Prettier formats, ESLint (specifically `typescript-eslint`) only checks things Prettier structurally cannot — type errors, unused variables, unsafe patterns — with all of ESLint's own stylistic/formatting rules turned off via `eslint-config-prettier`.

## Bad

```javascript
// eslint.config.js — ESLint's own formatting rules enabled alongside Prettier,
// so the two disagree about semicolons/quotes and every commit fights itself
export default [
  {
    rules: {
      indent: ['error', 2],
      quotes: ['error', 'single'],
      semi: ['error', 'always'],
    },
  },
];
```

## Good

```bash
npm install --save-dev prettier eslint-config-prettier
```

```javascript
// eslint.config.js
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import eslintConfigPrettier from 'eslint-config-prettier';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  eslintConfigPrettier, // must be last: disables all ESLint formatting rules
);
```

```jsonc
// .prettierrc.json
{
  "semi": true,
  "singleQuote": true,
  "trailingComma": "all",
  "printWidth": 100
}
```

```json
// package.json scripts
{
  "scripts": {
    "format": "prettier --write .",
    "format:check": "prettier --check .",
    "lint": "eslint ."
  }
}
```

## Division of Responsibility

| Concern | Owned by |
|---|---|
| Indentation, quotes, semicolons, line length | Prettier |
| Unused variables, floating promises, `any` usage | ESLint (`typescript-eslint`) |
| Import order/grouping | Either — commonly `eslint-plugin-import` or `perfectionist`, since Prettier doesn't sort imports |

Run `prettier --check` and `eslint` as two separate, both-required CI steps rather than trying to make one tool own both jobs.

## See Also

- [lint-typescript-eslint-recommended](lint-typescript-eslint-recommended.md) - Adopt `typescript-eslint`'s recommended (or recommended-type-checked) config
- [lint-ci-lint-gate](lint-ci-lint-gate.md) - Run typecheck and lint as a required CI gate
