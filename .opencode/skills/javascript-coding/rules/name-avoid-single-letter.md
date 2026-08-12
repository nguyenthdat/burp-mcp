# name-avoid-single-letter

> Avoid single-letter variable names except for loop indices (`i`, `j`, `k`)

## Why It Matters

Single-letter names carry zero semantic meaning. A variable named `x` could be anything — a coordinate, a count, a temporary value. Searching for single-letter names is impossible (they match everywhere). The few microseconds saved typing are dwarfed by the hours lost to confusion during debugging and code review.

## Bad

```js
// Single-letter variables — meaningless
function f(a, b) {
  const x = a * b;
  const y = x / 100;
  return y;
}

const d = users.map(u => u.name);
const r = await fetch(url);
const t = setTimeout(() => { }, 1000);
```

## Good

```js
// Descriptive names
function calculateDiscount(price, percent) {
  const discount = price * percent;
  const final = discount / 100;
  return final;
}

const userNames = users.map(user => user.name);
const response = await fetch(url);
const timer = setTimeout(() => { }, 1000);
```

## Acceptable Single-Letter Names

```js
// Loop indices — conventional and limited in scope
for (let i = 0; i < items.length; i++) {
  for (let j = 0; j < items[i].length; j++) {
    process(items[i][j]);
  }
}

// Coordinates in math/graphics code
function distance(x1, y1, x2, y2) {
  return Math.sqrt((x2 - x1) ** 2 + (y2 - y1) ** 2);
}

// Generic type parameters in generic functions (in typed JS)
function identity(v) { return v; }

// Destructured catch — conventional
try { /* ... */ } catch (e) {
  console.error(e);
}
// Better:
try { /* ... */ } catch (err) {
  console.error(err);
}
```

## When Exceptions Apply

In dense mathematical code, single-letter variables matching standard notation (`x`, `y`, `a`, `b`) are acceptable. In all other contexts, use descriptive names.

## See Also

- [name-camelCase](./name-camelCase.md) - camelCase convention
- [name-no-abbrev](./name-no-abbrev.md) - Avoid abbreviations
