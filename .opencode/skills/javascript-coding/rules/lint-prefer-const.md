# lint-prefer-const

> Use `const` by default — only use `let` when reassignment is needed

## Why It Matters

`const` signals that a binding won't be reassigned, making code easier to reason about. It prevents accidental reassignment bugs and enables JavaScript engine optimizations. If most variables are `const`, the few `let` declarations stand out as intentionally mutable, guiding code review.

## Bad

```js
// let when the value never changes — misleading
let name = 'Alice';
let count = 0;
let config = loadConfig();
let results = process(items);

// Reassignment used for different types — confusing
let value = 'hello';
value = 42;
value = { data: 'stuff' };
```

## Good

```js
// const for values that don't change binding
const name = 'Alice';
const count = items.length;
const config = loadConfig();
const results = process(items);

// let only when reassignment is needed
let retries = 0;
while (retries < 3) {
  try {
    await doWork();
    break;
  } catch {
    retries++;
  }
}

let status = 'pending';
if (condition) {
  status = 'approved';
}
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'prefer-const': 'error',
  },
}
```

## When Exceptions Apply

Loop counters (`let i = 0`) and accumulator variables are the primary uses of `let`. If you find yourself using `let` frequently, consider if the logic can be refactored to use `const` with immutable patterns.

## See Also

- [lint-no-var](./lint-no-var.md) - No var
- [fn-immutability](./fn-immutability.md) - Immutable patterns
