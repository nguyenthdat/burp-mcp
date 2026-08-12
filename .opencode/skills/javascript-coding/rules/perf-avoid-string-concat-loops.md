# perf-avoid-string-concat-loops

> Use `Array.join()` instead of `+=` string concatenation in loops

## Why It Matters

Strings in JavaScript are immutable. Each `+=` operation creates a new string by copying both the original and the appended content. In a loop of N iterations, this results in O(n²) character copying. `Array.join()` collects all parts in an array (O(n) memory) and joins them once (O(n)), avoiding repeated allocation.

## Bad

```js
// O(n²) string concatenation — creates a new string each iteration
function buildHTML(items) {
  let html = '';
  for (const item of items) {
    html += `<li>${item.name}</li>`;  // Creates a new string each time
  }
  return html;
}
// For 10,000 items: ~50 million character copies
```

## Good

```js
// O(n) — push to array, join once
function buildHTML(items) {
  const parts = [`<ul>`];
  for (const item of items) {
    parts.push(`<li>${item.name}</li>`);
  }
  parts.push('</ul>');
  return parts.join('');
}

// Or: use .map() + .join()
function buildHTML(items) {
  const listItems = items.map(item => `<li>${item.name}</li>`);
  return `<ul>${listItems.join('')}</ul>`;
}
```

## When Exceptions Apply

For loops with fewer than ~100 iterations, the performance difference is negligible. Use the most readable approach. For large data transformations (templates, reports, XML generation), use `Array.join()`.

## See Also

- [fn-template-literals](./fn-template-literals.md) - Template literals
- [fn-map-over-for](./fn-map-over-for.md) - Array methods
