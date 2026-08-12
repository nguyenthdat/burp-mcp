# api-named-over-default-export

> Prefer named exports over default exports

## Why It Matters

Default exports let every importer choose an arbitrary local name, which means the same value can be called `Foo`, `foo`, `MyComponent`, or anything else across a codebase — this defeats project-wide search/rename tooling and makes it harder to know, from an import statement alone, what's actually being imported. Named exports are refactor-safe (renaming the source automatically surfaces every call site via "rename symbol"), tree-shake more reliably in bundlers, and enforce a single canonical name across the entire codebase.

## Bad

```typescript
// format-currency.ts
export default function formatCurrency(amount: number, currency: string): string {
  return new Intl.NumberFormat("en-US", { style: "currency", currency }).format(amount);
}

// consumer-a.ts
import formatMoney from "./format-currency.js"; // renamed arbitrarily
// consumer-b.ts
import fmt from "./format-currency.js"; // renamed again, differently
```

## Good

```typescript
// format-currency.ts
export function formatCurrency(amount: number, currency: string): string {
  return new Intl.NumberFormat("en-US", { style: "currency", currency }).format(amount);
}

// consumer-a.ts and consumer-b.ts
import { formatCurrency } from "./format-currency.js"; // same name everywhere
```

## Why This Matters More in TypeScript Specifically

```typescript
// Default exports interact poorly with CommonJS interop settings.
// Depending on esModuleInterop / allowSyntheticDefaultImports / module
// resolution mode, `import Foo from "cjs-package"` may or may not work
// as expected, and mixing default+named exports from the same module
// creates ambiguity about what `import * as ns` produces.

// Named exports have none of this ambiguity: they compile to the same
// stable shape under every module target.
```

## When a Default Export Is Still Reasonable

- A framework convention requires it (e.g. Next.js `pages/`/`app/` route files, some bundler-based config files).
- The module exports exactly one thing and is always imported as a whole (rare in application code, more common in single-purpose scripts).

Outside of those cases, prefer named exports as the default choice.

## See Also

- [api-minimal-surface](api-minimal-surface.md) - Keep the public API surface as small as the consumer actually needs
- [api-barrel-file-tradeoffs](api-barrel-file-tradeoffs.md) - Use barrel (`index.ts`) files judiciously; they can defeat tree-shaking
- [proj-verbatim-module-syntax](proj-verbatim-module-syntax.md) - Configuring TypeScript's ESM/CJS interop correctly
