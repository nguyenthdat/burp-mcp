# lint-eslint-setup

> Use ESLint with the new flat config format (`eslint.config.mjs`)

## Why It Matters

ESLint's flat config (introduced in v8.21, default in v9) replaces the legacy `.eslintrc` format. It's simpler (no `extends` nesting), faster (no cascading config resolution), and the only format supported going forward. New projects should start with flat config; existing projects should migrate.

## Bad

```jsonc
// Legacy .eslintrc.json — deprecated format
{
  "extends": ["eslint:recommended"],
  "env": {
    "node": true,
    "es2024": true
  },
  "parserOptions": {
    "ecmaVersion": 2024,
    "sourceType": "module"
  },
  "rules": {
    "no-var": "error",
    "prefer-const": "error"
  }
}
```

## Good

```js
// eslint.config.mjs — flat config
import js from '@eslint/js';

export default [
  js.configs.recommended,
  {
    rules: {
      'no-var': 'error',
      'prefer-const': 'error',
      'eqeqeq': ['error', 'always'],
      'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      'no-console': 'warn',
      'max-params': ['warn', 4],
      'complexity': ['warn', 15],
    },
  },
  {
    ignores: ['dist/', 'coverage/', 'node_modules/'],
  },
];
```

## Migration

```bash
# Install ESLint v9+
npm install -D eslint@^9.0.0

# If you need to extend plugins
npm install -D eslint-plugin-import

# eslint.config.mjs
import js from '@eslint/js';
import importPlugin from 'eslint-plugin-import';

export default [
  js.configs.recommended,
  importPlugin.flatConfigs.recommended,
  { rules: { /* custom rules */ } },
];
```

## When Exceptions Apply

Legacy projects with complex `.eslintrc` setups should migrate gradually. Use `@eslint/migrate-config` to convert automatically.

## See Also

- [lint-no-var](./lint-no-var.md) - Use let/const
- [lint-prettier-format](./lint-prettier-format.md) - Prettier integration
