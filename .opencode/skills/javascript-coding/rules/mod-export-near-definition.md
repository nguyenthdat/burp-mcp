# mod-export-near-definition

> Export functions close to where they're defined — not at the bottom of the file

## Why It Matters

When exports are colocated with function definitions, it's immediately clear which functions are part of the module's public API. Scrolling to the bottom of a 200-line file to see exports creates cognitive overhead and makes refactoring error-prone. Inline `export` statements make the intent explicit at the point of definition.

## Bad

```js
// Definitions separated from exports
function calculateTax(price, rate) {
  return price * rate;
}

function applyDiscount(price, discount) {
  return price * (1 - discount);
}

function formatCurrency(amount) {
  return `$${amount.toFixed(2)}`;
}

// Bottom of file — 100 lines later
export { calculateTax, applyDiscount, formatCurrency };
```

## Good

```js
// Inline exports — clear intent at definition
export function calculateTax(price, rate) {
  return price * rate;
}

export function applyDiscount(price, discount) {
  return price * (1 - discount);
}

// Private helper — no export needed
function formatCurrency(amount) {
  return `$${amount.toFixed(2)}`;
}
```

## Grouped Re-exports for Large Modules

```js
// For modules with many exports, use barrel-style re-export at the bottom
// But prefer keeping the inline export on each definition

// If you must group:
function calculateTax(price, rate) { /* ... */ }
function applyDiscount(price, discount) { /* ... */ }
function getTaxRate(region) { /* ... */ }
function getDiscount(code) { /* ... */ }

export {
  calculateTax,
  applyDiscount,
  getTaxRate,
  getDiscount,
};
```

## When Exceptions Apply

Grouped exports at the bottom are acceptable when:
- The module is small (< 30 lines)
- It serves as a barrel re-export file
- You need to rename exports: `export { foo as bar }`

## See Also

- [mod-named-over-default](./mod-named-over-default.md) - Prefer named exports
- [mod-barrel-files](./mod-barrel-files.md) - Barrel file patterns
