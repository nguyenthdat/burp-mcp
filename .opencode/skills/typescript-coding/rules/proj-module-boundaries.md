# proj-module-boundaries

> Enforce module boundaries; don't import another module's internal files

## Why It Matters

When any file can `import` any other file regardless of folder structure, the codebase's real dependency graph becomes invisible — a "feature" boundary or "layer" boundary exists only as a convention, and conventions get violated the first time someone is in a hurry. Once `features/checkout` imports directly from `features/profile/components/internal/Avatar.tsx`, the two features are coupled at the file level: renaming or refactoring `Avatar.tsx` now risks breaking an unrelated feature, and there is no single "public API" for `profile` to review or version. Enforcing boundaries — via linting, not just code review — keeps internal restructuring safe and makes the intended architecture machine-checkable.

## Bad

```typescript
// features/checkout/components/OrderSummary.tsx
// Reaches past profile's public index.ts into its internals
import { formatAvatarUrl } from '../../profile/components/internal/avatar-utils';
```

## Good

```typescript
// features/profile/index.ts — the only sanctioned export surface
export { getAvatarUrl } from './components/internal/avatar-utils';
```

```typescript
// features/checkout/components/OrderSummary.tsx
import { getAvatarUrl } from '@features/profile';
```

## Configuration

Enforce this with `eslint-plugin-boundaries` or `import/no-internal-modules` so violations fail lint, not just review:

```jsonc
// .eslintrc.json (relevant excerpt)
{
  "plugins": ["boundaries"],
  "settings": {
    "boundaries/elements": [
      { "type": "feature", "pattern": "src/features/*" },
      { "type": "shared", "pattern": "src/shared/*" }
    ]
  },
  "rules": {
    "boundaries/no-private": "error",
    "boundaries/element-types": [
      "error",
      { "default": "disallow", "rules": [{ "from": "feature", "allow": ["shared"] }] }
    ]
  }
}
```

```jsonc
// Simpler alternative: forbid deep imports past a package's index
{
  "rules": {
    "import/no-internal-modules": [
      "error",
      { "allow": ["**/index", "@myorg/*"] }
    ]
  }
}
```

## See Also

- [proj-feature-based-structure](proj-feature-based-structure.md) - Organize source by feature/domain, not by technical file type
- [node-package-exports-map](node-package-exports-map.md) - Define package entry points with the `exports` field
- [api-minimal-surface](api-minimal-surface.md) - Expose the smallest public surface a module needs
