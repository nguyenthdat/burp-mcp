# api-versioned-public-api

> Version public package APIs deliberately and follow semver for breaking changes

## Why It Matters

Consumers of a published package (internal or on npm) pin dependency ranges based on the assumption that semver is honored: a patch release fixes bugs without changing behavior they depend on, a minor release only adds capability, and only a major release is allowed to break existing call sites. Silently shipping a breaking change — removing an export, tightening a parameter type, changing a return shape — as a patch or minor release breaks every consumer who trusted that contract, often without warning until their build fails in CI.

## Bad

```jsonc
// package.json — a type change that breaks callers ships as a patch
{
  "name": "@acme/pricing-utils",
  "version": "2.4.1" // was 2.4.0; this bump implies "just a bug fix"
}
```

```typescript
// v2.4.0
export function formatPrice(amountCents: number): string { /* ... */ }

// v2.4.1 — the parameter's *meaning* silently changed, still same signature
export function formatPrice(amountCents: number): string {
  // now expects amount in whole currency units, not cents — every
  // existing caller now produces wrong output, with no compile error
  // and a version bump that claimed "just a patch"
}
```

## Good

```jsonc
// package.json
{
  "name": "@acme/pricing-utils",
  "version": "3.0.0" // major bump: signal to every consumer that this needs review
}
```

```typescript
// Give the breaking change a new, distinctly-named export instead of
// silently repurposing an existing one, and deprecate the old one first.

/** @deprecated Use `formatPriceFromUnits` instead. Will be removed in 4.0. */
export function formatPrice(amountCents: number): string { /* unchanged */ }

export function formatPriceFromUnits(amountInCurrencyUnits: number): string {
  return formatPrice(Math.round(amountInCurrencyUnits * 100));
}
```

## Semver Discipline for TypeScript-Specific Changes

| Change | Semver bump |
|---|---|
| Widening a parameter type (accepting more than before) | Minor (backward compatible) |
| Narrowing a parameter type (accepting less than before) | Major (breaking) |
| Widening a return type (e.g. adding a new union member callers must now handle in an exhaustive switch) | Major (breaking for exhaustive consumers) |
| Narrowing a return type (returning a more specific subtype) | Minor (backward compatible for consumers, though it can break their own exhaustiveness checks in edge cases — document it) |
| Adding a new required parameter | Major |
| Adding a new optional parameter | Minor |
| Adding a new named export | Minor |
| Removing or renaming any export | Major |

## Communicating Deprecation Before Removal

Use `@deprecated` TSDoc tags (surfaced by editors as strikethrough) for at least one minor version cycle before actually removing the export in a major release, and record every breaking change in the changelog under a clear "BREAKING CHANGES" heading.

## See Also

- [doc-changelog-semver](doc-changelog-semver.md) - Maintaining a changelog that documents semver-relevant changes
- [doc-deprecated-tag](doc-deprecated-tag.md) - Using the `@deprecated` TSDoc tag before removing an API
- [api-minimal-surface](api-minimal-surface.md) - Keep the public API surface as small as the consumer actually needs
