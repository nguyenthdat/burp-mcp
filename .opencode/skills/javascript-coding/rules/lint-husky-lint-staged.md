# lint-husky-lint-staged

> Use `lint-staged` + `husky` for pre-commit linting and formatting checks

## Why It Matters

Code that fails linting or formatting should never reach the repository. Pre-commit hooks catch issues before they're committed, preventing broken CI builds and saving reviewer time. `lint-staged` runs linters only on staged files (fast), and `husky` manages git hooks without shell scripts.

## Bad

```bash
# Manual linting — easily forgotten
$ eslint .
$ prettier --write .
$ git commit -m "fix"
# No enforcement — broken code committed
```

## Good

```bash
npm install -D husky lint-staged
npx husky init
```

```jsonc
// package.json
{
  "scripts": {
    "prepare": "husky"
  },
  "lint-staged": {
    "*.{js,mjs,cjs}": ["eslint --fix", "prettier --write"],
    "*.{json,md,yaml}": ["prettier --write"]
  }
}
```

```bash
# .husky/pre-commit
npx lint-staged
```

## When Exceptions Apply

For solo projects, pre-commit hooks are optional but still beneficial. In CI-only setups, run linting as a CI check instead (no local hooks).

## See Also

- [lint-eslint-setup](./lint-eslint-setup.md) - ESLint configuration
- [lint-prettier-format](./lint-prettier-format.md) - Prettier setup
