# lint-unused-vars-error

> Treat unused variables as errors — they indicate bugs or dead code

## Why It Matters

Unused variables clutter the codebase, make refactoring harder, and sometimes indicate real bugs (e.g., forgetting to use a returned value). They're a form of dead code. ESLint's `no-unused-vars` catches them automatically. The `argsIgnorePattern: '^_'` convention allows intentional unused parameters.

## Bad

```js
// Unused variable — dead code or bug
import { unusedHelper } from './utils.js';  // Never used — dead import

function process(data, options) {
  const temp = calculate(data);  // temp never used — wasted computation
  return data;
}
```

## Good

```js
// Clean imports
import { helper } from './utils.js';

function process(data) {
  return helper(data);
}

// Intentional unused with underscore prefix
function handler(req, res, _next) {
  // _next is an Express middleware parameter we intentionally don't use
  res.json({ ok: true });
}
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'no-unused-vars': ['error', {
      argsIgnorePattern: '^_',
      varsIgnorePattern: '^_',
      caughtErrorsIgnorePattern: '^_',
    }],
  },
}
```

## When Exceptions Apply

Destructuring where you only need some properties: `const { id, ..._rest } = obj;` — the `_rest` convention signals intentional non-use.

## See Also

- [lint-eslint-setup](./lint-eslint-setup.md) - ESLint configuration
- [doc-no-stale-comments](./doc-no-stale-comments.md) - Remove dead code
