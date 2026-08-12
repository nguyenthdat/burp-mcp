# type-avoid-implicit-coercion

> Use `===` and `!==` over `==` and `!=` — avoid implicit type coercion

## Why It Matters

JavaScript's `==` operator performs type coercion before comparison, leading to surprising results: `0 == ''`, `null == undefined`, `' \t\r\n ' == 0`. These bugs are hard to spot in code review. Strict equality (`===`) compares both value and type, producing predictable, unsurprising results. There is almost never a good reason to use `==`.

## Bad

```js
// Surprising coercion with ==
if (value == 0) { /* ... */ }      // true for '', false, [], 0
if (items.length == '0') { /* ... */ }  // true
if (input == null) { /* ... */ }       // true for null AND undefined

// Implicit coercion in conditionals
if (count) { /* ... */ }  // false for 0 — maybe intentional, maybe bug
if (name) { /* ... */ }   // false for '' — probably a bug
```

## Good

```js
// Strict equality — predictable
if (value === 0) { /* ... */ }
if (items.length === 0) { /* ... */ }

// Explicit null/undefined checks
if (input === null) { /* ... */ }
if (input === undefined) { /* ... */ }
if (input == null) { /* ... */ }  // Only acceptable use of ==: catches both null and undefined

// Or use ?? for cleaner intent
const count = input.count ?? 0;
```

## Truthy/Falsy Pitfalls

```js
// Be explicit about what you're checking
// Bad
if (count) process(count);  // Skips count=0 — probably a bug

// Good
if (count > 0) process(count);        // Explicit numeric check
if (typeof count === 'number') process(count);  // Type check
if (count != null) process(count);    // Exclude null/undefined only
if (Array.isArray(items) && items.length > 0) process(items);  // Full check
```

## When Exceptions Apply

`value == null` is the one idiomatic use of `==` — it checks for both `null` and `undefined` in a single expression. This is widely accepted and preferred over `value === null || value === undefined`.

## See Also

- [lint-strict-comparisons](./lint-strict-comparisons.md) - ESLint eqeqeq rule
- [type-typeof-guards](./type-typeof-guards.md) - Type guards
