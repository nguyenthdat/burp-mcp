# lint-no-var

> Use `let` and `const` — never `var`

## Why It Matters

`var` has function scope (not block scope), is hoisted in confusing ways, allows redeclaration, and creates properties on the global object. `let` and `const` have block scope, no redeclaration, and predictable hoisting (temporal dead zone). There is zero reason to use `var` in modern JavaScript.

## Bad

```js
// var leaks out of blocks
if (condition) {
  var result = 'yes';  // Accessible outside the block!
}
console.log(result);  // 'yes' — surprising

// var redeclares silently
var x = 1;
var x = 2;  // No error

// var on global scope pollutes window/globalThis
var globalVar = 'leaked';
globalThis.globalVar;  // 'leaked'
```

## Good

```js
// let for reassignable variables
let result;
if (condition) {
  result = 'yes';  // Block-scoped
}
// result is 'yes' here

// const for non-reassignable bindings
const MAX_SIZE = 100;
const items = [];  // items.push() works, items = [] doesn't

// Temporal dead zone catches bugs
console.log(x);  // ReferenceError — caught early
let x = 1;
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'no-var': 'error',
  },
}
```

## When Exceptions Apply

There are no exceptions. `var` is obsolete. If you see `var` in legacy code, refactor to `let`/`const`.

## See Also

- [lint-prefer-const](./lint-prefer-const.md) - Prefer const over let
- [fn-immutability](./fn-immutability.md) - Immutable patterns
