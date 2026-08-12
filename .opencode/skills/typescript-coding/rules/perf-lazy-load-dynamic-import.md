# perf-lazy-load-dynamic-import

> Use dynamic `import()` for code splitting and lazy loading

## Why It Matters

Statically importing every module a page could ever need — a rarely used admin panel, a PDF export feature, a heavy charting library — means all of that code ships in the initial bundle and blocks the first paint, even for users who never touch those features. Dynamic `import()` lets a bundler split that code into a separate chunk that only downloads when (and if) it's actually needed, directly cutting time-to-interactive for the common path.

## Bad

```typescript
// app.ts — the charting library ships in every page load,
// even though most users never open the analytics tab.
import { renderChart } from "heavy-charting-library";

export function initApp() {
  document.getElementById("analytics-tab")?.addEventListener("click", () => {
    renderChart(document.getElementById("chart-container")!);
  });
}
```

## Good

```typescript
// app.ts — the charting library only loads when the user opens the tab
export function initApp() {
  document.getElementById("analytics-tab")?.addEventListener("click", async () => {
    const { renderChart } = await import("heavy-charting-library");
    renderChart(document.getElementById("chart-container")!);
  });
}
```

```typescript
// React: lazy-load a route-level component
import { lazy, Suspense } from "react";

const AdminPanel = lazy(() => import("./AdminPanel"));

function App() {
  return (
    <Suspense fallback={<Spinner />}>
      <AdminPanel />
    </Suspense>
  );
}
```

## Common Patterns

- Split at route boundaries first (each page/route as its own chunk) — this is the highest-leverage place to apply lazy loading in most apps.
- Split heavy, rarely used libraries (rich text editors, chart libraries, PDF generators) behind the interaction that needs them, not behind the initial render.
- Prefetch a likely-next chunk on hover/idle with `import(/* webpackPrefetch: true */ "./NextPage")` to hide the network latency before the user actually navigates.
- Verify the split actually happened by inspecting the built output (`vite build` chunk report, webpack-bundle-analyzer) — a dynamic import can still get merged back into the main chunk if the bundler decides it's not worth splitting for a tiny module.
- For Node.js backends, dynamic `import()` is also useful for deferring the cost of loading rarely used, heavy dependencies (e.g. a PDF renderer only needed by one endpoint) until the first request that needs them.

## See Also

- [perf-bundle-size-audit](perf-bundle-size-audit.md) - Audit bundle size and dependency weight regularly
- [perf-tree-shaking-friendly](perf-tree-shaking-friendly.md) - Write side-effect-free modules so bundlers can tree-shake unused exports
- [async-top-level-await](async-top-level-await.md) - related async module-loading considerations
