# fn-nullish-coalescing

> Prefer `??` over `||` for default values when 0, '', and false are valid values

## Why It Matters

JavaScript's `||` operator treats ALL falsy values (`0`, `''`, `false`, `NaN`, `null`, `undefined`) as triggers for the default. This causes subtle bugs when falsy-but-valid values are overwritten. `??` (nullish coalescing) only falls back on `null` and `undefined`, making it the correct choice for defaults in most cases.

## Bad

```js
// || overwrites valid falsy values
const pageSize = userInput.pageSize || 10;    // 0 becomes 10 — bug
const username = user.name || 'Anonymous';     // '' becomes 'Anonymous'
const isEnabled = config.isEnabled || true;   // false becomes true
const count = data.count || -1;               // 0 becomes -1
```

## Good

```js
// ?? only catches null and undefined
const pageSize = userInput.pageSize ?? 10;     // 0 stays 0
const username = user.name ?? 'Anonymous';      // '' stays ''
const isEnabled = config.isEnabled ?? true;    // false stays false
const count = data.count ?? -1;                // 0 stays 0
```

## When To Use Each

```js
// Use ?? for defaults (null/undefined only)
const name = input.name ?? 'default';

// Use || for boolean-like checks (any falsy)
const greeting = name || 'Stranger';  // '' is not a valid name either

// Use ternary for explicit null checks
const result = value !== null ? value : fallback;

// Chained defaults with ??
const host = config.host ?? process.env.HOST ?? 'localhost';
```

## Common Pitfall: Mixing Operators

```js
// Don't mix ?? and || without parentheses — syntax error or unexpected behavior
// const x = a ?? b || c;  // Error: cannot mix ?? with &&/|| without parens

// Use parentheses if you must mix
const x = (a ?? b) || c;
```

## When Exceptions Apply

Use `||` when you genuinely want to catch ALL falsy values (e.g., providing a fallback string for empty input). This is less common than `??` use cases.

## See Also

- [fn-optional-chaining](./fn-optional-chaining.md) - ?. for safe property access
- [fn-default-params](./fn-default-params.md) - Default function parameters
