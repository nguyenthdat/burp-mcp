# lint-strict-comparisons

> Enforce `===` and `!==` over `==` and `!=`

## Why It Matters

Loose equality (`==`) performs type coercion, producing surprising results: `0 == ''`, `null == undefined`, `' \t\r\n ' == 0`. These bugs are notoriously hard to spot in code review. Strict equality (`===`) compares value AND type, producing predictable results. ESLint's `eqeqeq` rule enforces this.

## Bad

```js
// Loose equality — surprising coercion
if (value == 0) { }       // true for '', false, [], 0
if (items.length == '0') { }  // true
if (input != null) { }        // catches undefined too — intentional but confusing
```

## Good

```js
// Strict equality — predictable
if (value === 0) { }
if (items.length === 0) { }

// Explicit null/undefined checks
if (input !== null && input !== undefined) { }
// Or use the one acceptable use of ==:
if (input == null) { }  // Catches both null and undefined
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'eqeqeq': ['error', 'always', { null: 'ignore' }],  // Allow == null
  },
}
```

## When Exceptions Apply

`value == null` is the one idiomatic use of `==` — it checks for both `null` and `undefined` in one expression. Most ESLint configs allow this exception.

## See Also

- [type-avoid-implicit-coercion](./type-avoid-implicit-coercion.md) - Avoid implicit coercion
- [fn-nullish-coalescing](./fn-nullish-coalescing.md) - ?? for null/undefined
