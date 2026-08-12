# perf-bundle-size-audit

> Audit bundle size and dependency weight regularly

## Why It Matters

Every dependency added to a frontend project ships to every user on every page load, and bundle size grows silently — a single "just add this small helper library" decision can pull in a large dependency tree, and nobody notices until the app is measurably slower to load. Auditing bundle size regularly (not just at launch) catches this drift early, when it's a five-minute fix (swap a heavy dependency for a smaller one), instead of after months of accumulation, when it's a multi-day refactor.

## Bad

```typescript
// package.json — added without checking size, for one function
{
  "dependencies": {
    "moment": "^2.30.0",       // ~70KB minified, for `moment().format()` in one place
    "lodash": "^4.17.21"       // ~70KB minified when imported as a whole package
  }
}
```

```typescript
// Imports the entire lodash package for one function
import _ from "lodash";
const unique = _.uniq(items);
```

## Good

```typescript
// Use modern, tree-shakeable, or native alternatives
const formatted = new Intl.DateTimeFormat("en-US").format(date); // no dependency

// Import only the specific function needed, from a tree-shakeable build
import uniq from "lodash-es/uniq";
const unique = uniq(items);

// Or replace with a one-line native implementation
const unique = [...new Set(items)];
```

## Auditing Workflow

- Run `npx bundlephobia <package-name>` or check bundlephobia.com before adding a new dependency, to see its minified + gzipped size and whether it's tree-shakeable.
- Wire `size-limit` (or `bundlewatch`) into CI with a hard budget per entry point; fail the build if a PR pushes the bundle over the threshold.
- Periodically run `webpack-bundle-analyzer` or `source-map-explorer` on the production build to visualize what's actually taking up space — dependencies often look small in isolation but bring in surprising transitive weight.
- Prefer native platform APIs (`Intl`, `structuredClone`, `Array.prototype` methods) over utility libraries where the native API now covers the same use case — much of what libraries like lodash and moment provided is now built into the language and browsers.
- Re-audit after major dependency upgrades; a minor version bump can occasionally add new transitive dependencies.

## See Also

- [perf-tree-shaking-friendly](perf-tree-shaking-friendly.md) - Write side-effect-free modules so bundlers can tree-shake unused exports
- [perf-lazy-load-dynamic-import](perf-lazy-load-dynamic-import.md) - Use dynamic `import()` for code splitting and lazy loading
- [perf-avoid-premature-optimize](perf-avoid-premature-optimize.md) - Profile before optimizing
