# proj-feature-based-structure

> Organize source by feature/domain, not by technical file type

## Why It Matters

Organizing by technical layer (`components/`, `hooks/`, `services/`, `reducers/`) scatters everything related to a single feature across the tree, so adding or removing a feature means editing five unrelated top-level folders and grepping to find every piece. Organizing by feature/domain instead (`features/checkout/`, `features/profile/`) keeps everything one feature needs — components, hooks, API calls, types, tests — in one place, so the folder structure mirrors how people actually think and work: "I'm changing checkout" rather than "I'm changing components, and also hooks, and also services." It also makes module boundaries visible in the filesystem, which supports enforcing that features only talk to each other through explicit public exports (see `proj-module-boundaries`).

## Bad

```
src/
  components/
    PaymentForm.tsx
    ProfileForm.tsx
    CartSummary.tsx
  hooks/
    useCheckout.ts
    useProfile.ts
  services/
    checkoutApi.ts
    profileApi.ts
  types/
    checkout.ts
    profile.ts
```

## Good

```
src/
  features/
    checkout/
      components/
        PaymentForm.tsx
        CartSummary.tsx
      hooks/
        useCheckout.ts
      api/
        checkoutApi.ts
      types.ts
      index.ts        # public surface of this feature
    profile/
      components/
        ProfileForm.tsx
      hooks/
        useProfile.ts
      api/
        profileApi.ts
      types.ts
      index.ts
  shared/
    ui/
    utils/
```

```typescript
// features/checkout/index.ts — the only sanctioned import surface
export { CartSummary } from './components/CartSummary';
export { useCheckout } from './hooks/useCheckout';
export type { CheckoutState } from './types';
```

## When To Still Split By Type

Very small features (a handful of files) don't need internal `components/`/`hooks/` subfolders — flatten them. And genuinely cross-cutting, feature-agnostic code (a design system, a date-formatting utility) belongs in a `shared/` folder, not duplicated per feature. The goal is that a feature's folder answers "what does this do", not "what kind of file is this."

## See Also

- [proj-module-boundaries](proj-module-boundaries.md) - Enforce module boundaries; don't import another module's internal files
- [proj-colocate-tests](proj-colocate-tests.md) - Colocate tests with source, or mirror source structure consistently — pick one
- [proj-path-aliases](proj-path-aliases.md) - Use `tsconfig` path aliases instead of long relative import chains
- [api-minimal-surface](api-minimal-surface.md) - Expose the smallest public surface a module needs
