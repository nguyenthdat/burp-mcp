# anti-var-usage

> Don't use `var`; use `const`/`let`

## Why It Matters

`var` is function-scoped (or globally-scoped), not block-scoped, and is hoisted with its declaration initialized to `undefined` before the assignment line runs — both properties routinely produce bugs that `const`/`let` structurally prevent. A `var` declared inside an `if` block or a `for` loop leaks out into the enclosing function, silently shadowing or clobbering a variable of the same name elsewhere, and closures captured over a `var` loop variable all see the *same* final value rather than the value at each iteration. `let`/`const` are block-scoped and are not accessible before their declaration (the "temporal dead zone" throws a clear error instead of silently returning `undefined`), which converts a whole category of hoisting bugs into either a compile-time or an immediate runtime error.

## Bad

```typescript
function processItems(items: string[]) {
  for (var i = 0; i < items.length; i++) {
    setTimeout(() => {
      console.log(items[i]); // logs items[items.length] (undefined) for every callback —
    }, 100);                 // all closures share the same `var i`
  }

  if (items.length > 0) {
    var first = items[0];
  }
  console.log(first); // `var` leaked out of the `if` block; works, but fragile
}
```

## Good

```typescript
function processItems(items: string[]) {
  for (let i = 0; i < items.length; i++) {
    setTimeout(() => {
      console.log(items[i]); // each closure captures its own `i` — correct per-iteration value
    }, 100);
  }

  let first: string | undefined;
  if (items.length > 0) {
    first = items[0];
  }
  console.log(first); // scoping is explicit and intentional
}
```

## Enforcement

```javascript
// eslint.config.js
export default [{ rules: { 'no-var': 'error', 'prefer-const': 'error' } }];
```

`no-var` bans `var` outright; `prefer-const` further nudges any `let` that's never reassigned to become a `const`, since an unreassigned binding declared with `let` is a missed signal to the reader that the value is fixed.

## See Also

- [imm-prefer-const](imm-prefer-const.md) - Prefer `const` bindings for values that never change
- [fn-early-return](fn-early-return.md) - Use early returns to avoid deep nesting
- [async-immediately-invoked](async-immediately-invoked.md) - Use IIFEs correctly for scoping in async contexts
