# proj-path-aliases

> Use `tsconfig` path aliases instead of long relative import chains

## Why It Matters

Deep relative imports like `../../../../shared/utils/format` are fragile: moving either the importing file or the imported file breaks every reference, and reviewers can't tell at a glance whether an import crosses a module boundary or stays local. Path aliases (`@shared/utils/format`) decouple the import specifier from the file's physical location, so refactors that move files don't cascade into unrelated diffs, and imports become self-describing about which layer of the codebase they reach into. The tradeoff is that aliases must be configured consistently across the compiler, the bundler, and the test runner, or you get "works in editor, fails at build" bugs.

## Bad

```typescript
// src/features/checkout/components/PaymentForm.tsx
import { formatCurrency } from '../../../../shared/utils/format';
import { useAuth } from '../../../../shared/hooks/useAuth';
import { Button } from '../../../../shared/ui/Button';
```

## Good

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@shared/*": ["src/shared/*"],
      "@features/*": ["src/features/*"]
    }
  }
}
```

```typescript
// src/features/checkout/components/PaymentForm.tsx
import { formatCurrency } from '@shared/utils/format';
import { useAuth } from '@shared/hooks/useAuth';
import { Button } from '@shared/ui/Button';
```

## Wiring Aliases Into Your Toolchain

`tsconfig.json` path mapping only affects type-checking; it does not rewrite import specifiers at runtime or in a bundler. Each tool needs its own matching configuration:

```jsonc
// vite.config.ts (conceptually — use vite-tsconfig-paths or resolve.alias)
{
  "resolve": {
    "alias": {
      "@shared": "/src/shared",
      "@features": "/src/features"
    }
  }
}
```

```jsonc
// vitest.config.ts / jest.config.ts need the same mapping, e.g. via
// the `vite-tsconfig-paths` plugin or Jest's `moduleNameMapper`
```

For Node.js execution without a bundler, prefer subpath imports (`#shared/*` in `package.json`'s `imports` field) over `tsconfig` paths, since Node natively resolves those without extra tooling.

## See Also

- [proj-module-boundaries](proj-module-boundaries.md) - Enforce module boundaries; don't import another module's internal files
- [proj-feature-based-structure](proj-feature-based-structure.md) - Organize source by feature/domain, not by technical file type
- [proj-single-tsconfig-base](proj-single-tsconfig-base.md) - Share a base `tsconfig.json` and extend it per package
