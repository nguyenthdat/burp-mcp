# type-strict-null-checks

> Enable `strictNullChecks` and model absence with `undefined`/`null` explicitly

## Why It Matters

Without `strictNullChecks`, every type implicitly includes `null` and `undefined`, so the compiler cannot warn you when you access a property on a value that might be absent — the classic "cannot read properties of undefined" crash. Turning it on (it's included in `strict: true`) forces every possibly-absent value to be explicitly typed as `T | undefined` or `T | null`, and forces you to handle that case before use, moving an entire category of runtime crashes to compile time.

## Bad

```typescript
// tsconfig without strictNullChecks
function getDiscount(user: User): number {
  // user.membership might be undefined at runtime, but the type says User
  return user.membership.discountPercent; // no compiler warning, crashes if membership is absent
}

interface User {
  name: string;
  membership: Membership; // lies about always being present
}
```

## Good

```typescript
// tsconfig.json: { "compilerOptions": { "strict": true } }
interface User {
  name: string;
  membership: Membership | undefined;
}

function getDiscount(user: User): number {
  if (user.membership === undefined) {
    return 0;
  }
  return user.membership.discountPercent; // narrowed, safe
}

// Optional chaining + nullish coalescing make the common case terse
function getDiscountShort(user: User): number {
  return user.membership?.discountPercent ?? 0;
}
```

## Configuration

```json
{
  "compilerOptions": {
    "strict": true,
    "strictNullChecks": true,
    "noUncheckedIndexedAccess": true
  }
}
```

`strict` already implies `strictNullChecks`, but enabling it explicitly documents intent, and it's worth knowing it's the specific flag responsible for this behavior when auditing an existing `tsconfig.json` that has cherry-picked flags rather than `strict: true`.

## Migrating a Legacy Codebase

Turning this flag on in a large, previously-loose codebase surfaces many errors at once. A common strategy:

1. Enable `strictNullChecks` at the tsconfig level.
2. Use `// @ts-expect-error` (not `@ts-ignore`) on each newly-surfaced error as a temporary, trackable marker.
3. Fix files incrementally, removing `@ts-expect-error` comments as you go; CI fails if an `@ts-expect-error` stops being necessary, so cleanup is enforced automatically.

## See Also

- [type-index-signature-safety](type-index-signature-safety.md) - Enable noUncheckedIndexedAccess and guard indexed access results
- [fn-optional-chaining](fn-optional-chaining.md) - Use optional chaining to safely access nested properties
- [fn-nullish-coalescing](fn-nullish-coalescing.md) - Use nullish coalescing for default values
- [lint-strict-tsconfig](lint-strict-tsconfig.md) - Enforce strict compiler options across the project
