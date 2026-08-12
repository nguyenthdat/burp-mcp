# api-barrel-file-tradeoffs

> Use barrel (`index.ts`) files judiciously; they can defeat tree-shaking

## Why It Matters

A barrel file that re-exports everything from a directory (`export * from "./thing.js"`) is convenient for consumers, but it forces most bundlers to load and evaluate the entire module graph behind the barrel just to resolve a single named import, unless the bundler's tree-shaking is unusually good and every module involved is provably side-effect-free. In large codebases this has caused measurable build-time and bundle-size regressions (Vite, webpack, and Next.js have all published guidance about barrel files degrading cold-start and bundle size), and it also makes circular-import bugs more likely because everything funnels through one aggregation point.

## Bad

```typescript
// components/index.ts — a barrel re-exporting the entire directory
export * from "./Button.js";
export * from "./Modal.js";
export * from "./DataTable.js"; // pulls in a large charting dependency
export * from "./VideoPlayer.js"; // pulls in a large video codec dependency

// consumer.ts
import { Button } from "./components/index.js";
// Depending on the bundler, this can still cause DataTable's and
// VideoPlayer's modules (and their heavy dependencies) to be
// evaluated or included, even though only Button is used.
```

## Good

```typescript
// consumer.ts — import directly from the specific module
import { Button } from "./components/Button.js";

// If you still want a barrel for developer convenience, keep it thin
// and be aware of the cost, or generate it for a subpackage boundary
// only (not for an entire large feature directory).
```

## When a Barrel Is Still Worth It

| Situation | Recommendation |
|---|---|
| Small, cohesive utility folder (a handful of small pure functions) | Barrel is fine; low re-export cost |
| Public package entry point (`src/index.ts` for an npm package) | Barrel is expected and idiomatic — it *is* the package's public API |
| Large feature directory with heavy, independent submodules | Avoid a barrel; import directly from the specific file |
| Directory with side-effectful modules (e.g. ones that register listeners on import) | Avoid a barrel; re-exporting can trigger those side effects unexpectedly |

## Mitigating the Cost With `sideEffects`

```jsonc
// package.json — tells bundlers which files have no import-time side effects
{
  "sideEffects": false
}
```

Setting `"sideEffects": false` (or listing the specific files that *do* have side effects) lets bundlers safely drop unused re-exports from a barrel, substantially reducing the tree-shaking penalty.

## See Also

- [api-named-over-default-export](api-named-over-default-export.md) - Prefer named exports over default exports
- [perf-tree-shaking-friendly](perf-tree-shaking-friendly.md) - Writing modules that bundlers can tree-shake effectively
- [perf-bundle-size-audit](perf-bundle-size-audit.md) - Periodically auditing bundle size for regressions
