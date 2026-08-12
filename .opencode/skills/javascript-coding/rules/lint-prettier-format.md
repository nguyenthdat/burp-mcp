# lint-prettier-format

> Use Prettier for opinionated, consistent code formatting

## Why It Matters

Formatting debates in code review waste time and create inconsistent code. Prettier is an opinionated formatter with few options, eliminating formatting decisions entirely. It integrates with ESLint (via `eslint-config-prettier` to disable conflicting rules) and editors (format-on-save). All code looks like one person wrote it.

## Bad

```js
// Inconsistent formatting across the codebase
function foo(x){return x*2}
const obj = {a:1,b:2,c:3};
if(condition)
  doSomething();
```

## Good

```js
// Prettier-formatted — consistent
function foo(x) {
  return x * 2;
}

const obj = { a: 1, b: 2, c: 3 };

if (condition) {
  doSomething();
}
```

## Configuration

```jsonc
// .prettierrc
{
  "singleQuote": true,
  "trailingComma": "all",
  "semi": true,
  "printWidth": 100,
  "tabWidth": 2
}
```

```jsonc
// package.json
{
  "scripts": {
    "format": "prettier --write .",
    "format:check": "prettier --check ."
  }
}
```

## When Exceptions Apply

Prettier has minimal configuration by design. Use `.prettierignore` for generated files, build output, and vendored code. Don't add ESLint formatting rules — use `eslint-config-prettier` to disable them.

## See Also

- [lint-eslint-setup](./lint-eslint-setup.md) - ESLint flat config
- [lint-husky-lint-staged](./lint-husky-lint-staged.md) - Pre-commit formatting
