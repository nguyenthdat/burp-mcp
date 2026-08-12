# api-readonly-public-types

> Mark public interface properties `readonly` unless mutation is part of the contract

## Why It Matters

An interface property without `readonly` implicitly tells consumers that reassigning it is a supported operation, even if your implementation never expected callers to do that. Marking properties `readonly` documents the intended contract directly in the type and makes TypeScript reject accidental mutation at every call site, catching bugs where a shared object is mutated by one consumer and silently corrupts state for another. It costs nothing at runtime — it's purely a compile-time guarantee — so there's rarely a reason not to default to it for data that flows across a public boundary.

## Bad

```typescript
export interface UserSession {
  userId: string;
  roles: string[];
  expiresAt: Date;
}

function checkAccess(session: UserSession) {
  // Nothing stops this from mutating a session another part of the
  // app is still relying on being unchanged.
  session.roles.push("admin");
}
```

## Good

```typescript
export interface UserSession {
  readonly userId: string;
  readonly roles: readonly string[];
  readonly expiresAt: Date;
}

function checkAccess(session: UserSession) {
  session.roles.push("admin"); // compile error: Property 'push' does not exist on type 'readonly string[]'
  // Must produce a new session instead:
  return { ...session, roles: [...session.roles, "admin"] };
}
```

## `readonly` Only Prevents Reassignment of the Reference, Not Deep Mutation

```typescript
interface Config {
  readonly limits: { readonly maxRequests: number };
}

function example(config: Config) {
  config.limits = { maxRequests: 10 }; // compile error, readonly
  config.limits.maxRequests = 10;      // ALSO a compile error here because
                                        // `limits` itself is typed with a readonly property
}
```

`readonly` must be applied at every level of a nested structure to get full protection — TypeScript's `Readonly<T>` utility type only makes the top level readonly, so for deep immutability use a recursive `DeepReadonly<T>` helper or a library like `type-fest`'s `ReadonlyDeep<T>`.

## Applying It in Bulk

```typescript
import type { ReadonlyDeep } from "type-fest";

interface RawConfig {
  server: { host: string; port: number };
  features: string[];
}

type Config = ReadonlyDeep<RawConfig>;
```

## See Also

- [imm-deep-immutability-types](imm-deep-immutability-types.md) - Using `type-fest`'s `ReadonlyDeep` and similar deep-immutability types
- [imm-readonly-class-fields](imm-readonly-class-fields.md) - Marking class fields `readonly` where mutation isn't part of the contract
- [type-readonly-arrays](type-readonly-arrays.md) - Preferring `readonly T[]` over `T[]` in function signatures
