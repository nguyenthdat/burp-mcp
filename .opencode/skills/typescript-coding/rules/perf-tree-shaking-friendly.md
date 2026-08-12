# perf-tree-shaking-friendly

> Write side-effect-free modules so bundlers can tree-shake unused exports

## Why It Matters

Bundlers like Rollup, esbuild, and webpack can only remove an unused export if they can prove importing the module has no observable side effect; a single top-level statement that mutates a global, registers something, or logs on import forces the bundler to keep the entire module (and its transitive imports) in the final bundle "just in case," even if the consumer only imported one small function. This can bloat a bundle by hundreds of kilobytes for a library where only one helper was actually used.

## Bad

```typescript
// utils/index.ts
console.log("utils module loaded"); // top-level side effect

export function formatCurrency(cents: number): string {
  return (cents / 100).toFixed(2);
}

export function formatDate(date: Date): string {
  return date.toISOString();
}

// A consumer that only needs formatCurrency still pulls in this entire
// file (and anything it imports) because the bundler can't prove the
// console.log is safe to drop.
```

## Good

```typescript
// utils/currency.ts — no top-level side effects
export function formatCurrency(cents: number): string {
  return (cents / 100).toFixed(2);
}

// utils/date.ts — separate module, separate side-effect analysis
export function formatDate(date: Date): string {
  return date.toISOString();
}
```

```json
// package.json — declare the package itself as side-effect-free
{
  "name": "@acme/utils",
  "sideEffects": false
}
```

## Configuration

- Set `"sideEffects": false` in `package.json` when no module in the package has import-time side effects; bundlers trust this flag to safely tree-shake even without deep static analysis.
- If specific files *do* have real side effects (e.g. a CSS import, a polyfill), list them explicitly: `"sideEffects": ["./src/polyfills.ts", "*.css"]`.
- Avoid class-based singletons instantiated at module scope (`export const client = new ApiClient()`) in shared libraries — they force the whole class graph to load; export a factory function instead.
- Prefer many small, focused modules over one large barrel file for anything performance-sensitive (see `api-barrel-file-tradeoffs`) — barrels can accidentally re-introduce side effects and make per-export tree-shaking harder for some bundlers.
- Verify tree-shaking actually worked with `source-map-explorer` or Rollup's `--experimental-bundleSize` output rather than assuming it did.

## See Also

- [perf-bundle-size-audit](perf-bundle-size-audit.md) - Audit bundle size and dependency weight regularly
- [api-barrel-file-tradeoffs](api-barrel-file-tradeoffs.md) - weigh the tree-shaking cost of barrel files
- [perf-lazy-load-dynamic-import](perf-lazy-load-dynamic-import.md) - Use dynamic `import()` for code splitting and lazy loading
