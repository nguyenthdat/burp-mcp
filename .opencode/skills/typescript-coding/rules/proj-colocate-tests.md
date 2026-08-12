# proj-colocate-tests

> Colocate tests with source, or mirror source structure consistently — pick one

## Why It Matters

Some teams put `*.test.ts` next to the file it tests; others keep a parallel `tests/` or `__tests__/` tree mirroring `src/`. Either is defensible, but mixing both in the same repo means nobody knows where to look for or add a test, coverage tools have to be configured for two patterns, and moving a source file silently orphans its test (if colocated) or leaves it in the wrong mirrored path (if separate). Pick one convention, encode it in tooling config so it's enforced rather than just documented, and apply it uniformly — the specific choice matters far less than the consistency.

## Bad

```
src/
  features/checkout/PaymentForm.tsx
  features/checkout/PaymentForm.test.tsx   # colocated here...
tests/
  features/profile/ProfileForm.test.tsx    # ...but mirrored there
__tests__/
  utils/format.test.ts                     # ...and yet a third pattern here
```

## Good — Option A: Colocated

```
src/
  features/checkout/
    PaymentForm.tsx
    PaymentForm.test.tsx
    useCheckout.ts
    useCheckout.test.ts
```

```json
// vitest.config.ts
{ "test": { "include": ["src/**/*.test.{ts,tsx}"] } }
```

## Good — Option B: Mirrored Tree

```
src/
  features/checkout/PaymentForm.tsx
  features/checkout/useCheckout.ts
tests/
  features/checkout/PaymentForm.test.tsx
  features/checkout/useCheckout.test.ts
```

```json
// vitest.config.ts
{ "test": { "include": ["tests/**/*.test.{ts,tsx}"], "root": "." } }
```

## Tradeoffs

| Approach | Pros | Cons |
|---|---|---|
| Colocated | Test is impossible to "forget" moving; obvious where to add one; short import paths in the test file | Source folders get noisier; some teams don't want test files shipped near source in the published package |
| Mirrored | Source tree stays test-free; easy to exclude `tests/` from a package build wholesale | Easy for a test to silently go stale/orphaned after a rename; deeper relative imports in the test |

Most TypeScript teams colocate for application code and mirror for published libraries (so the test tree isn't shipped). Whichever you choose, enforce it with an ESLint rule or a CI check that flags test files outside the sanctioned location.

## See Also

- [proj-feature-based-structure](proj-feature-based-structure.md) - Organize source by feature/domain, not by technical file type
- [test-isolate-tests](test-isolate-tests.md) - Keep tests isolated from each other
- [test-vitest-jest-setup](test-vitest-jest-setup.md) - Configure Vitest/Jest consistently across the repo
