# anti-loose-equality

> Don't use `==`/`!=`; use `===`/`!==`

## Why It Matters

`==` and `!=` perform type coercion before comparing, following a set of rules (documented in the ECMAScript spec's "Abstract Equality Comparison") that are notoriously non-intuitive: `0 == ''` is `true`, `'0' == false` is `true`, `null == undefined` is `true` but `null == 0` is `false`. These surprises turn a simple equality check into a source of subtle bugs that only manifest for specific edge-case inputs, and every reader has to re-derive the coercion rules to know whether a given `==` comparison is safe. `===`/`!==` compare value and type without coercion, which is both what most comparisons actually intend and what a reader can reason about without memorizing a coercion table.

## Bad

```typescript
function isEmpty(value: string | number) {
  return value == ''; // coerces: isEmpty(0) is true, which is likely unintended
}

if (userId == null) { // works, but hides which of null/undefined is meant
  redirectToLogin();
}

const count = '5';
if (count == 5) { // true due to coercion; masks that count is a string, not a number
  proceed();
}
```

## Good

```typescript
function isEmpty(value: string | number) {
  return value === ''; // exact comparison; isEmpty(0) is now false as intended
}

if (userId === null || userId === undefined) { // explicit about both cases
  redirectToLogin();
}

const count = Number('5');
if (count === 5) { // compares numbers to numbers, no coercion surprises
  proceed();
}
```

## The One Sanctioned Exception

```typescript
// Checking for both null and undefined at once is the one place == is
// conventionally allowed, and even then `?? ` or explicit checks are clearer
if (value == null) { // equivalent to `value === null || value === undefined`
  return defaultValue;
}

// Prefer the nullish coalescing operator instead, where applicable
const resolved = value ?? defaultValue;
```

Enforce this with ESLint's `eqeqeq` rule set to `"always"` (or `"smart"` to allow the `== null` idiom), so the rule is checked mechanically rather than relying on review discipline.

```javascript
// eslint.config.js
export default [{ rules: { eqeqeq: ['error', 'smart'] } }];
```

## See Also

- [fn-nullish-coalescing](fn-nullish-coalescing.md) - Use `??` instead of manual null/undefined checks
- [type-narrow-guards](type-narrow-guards.md) - Use type guards to narrow union types safely
- [anti-var-usage](anti-var-usage.md) - Don't use `var`; use `const`/`let`
