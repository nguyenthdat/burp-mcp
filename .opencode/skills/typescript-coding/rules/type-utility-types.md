# type-utility-types

> Prefer built-in utility types (`Pick`, `Omit`, `Partial`, `Required`) over hand-rolled equivalents

## Why It Matters

Manually rewriting a subset or variant of an existing interface duplicates its field list, so the two definitions inevitably drift apart the moment one is edited without the other. Built-in utility types derive the new shape mechanically from the original type, so adding, renaming, or removing a field in the source type automatically propagates everywhere it's referenced, with the compiler catching any place that's now inconsistent.

## Bad

```typescript
interface User {
  id: string;
  name: string;
  email: string;
  passwordHash: string;
  createdAt: Date;
}

// Hand-duplicated subset — drifts from User silently when User changes
interface PublicUser {
  id: string;
  name: string;
  email: string;
}

// Hand-duplicated "all optional" variant
interface UserUpdate {
  id?: string;
  name?: string;
  email?: string;
  passwordHash?: string;
  createdAt?: Date;
}
```

## Good

```typescript
interface User {
  id: string;
  name: string;
  email: string;
  passwordHash: string;
  createdAt: Date;
}

// Derived — adding a field to User is automatically reflected (or explicitly omitted)
type PublicUser = Omit<User, "passwordHash">;

type UserUpdate = Partial<Omit<User, "id" | "createdAt">>;

// Required inverts Partial when you need to force every optional field to be present
interface DraftPost {
  title?: string;
  body?: string;
}
type PublishedPost = Required<DraftPost>;
```

## Common Built-In Utility Types

| Utility | Effect |
|---|---|
| `Partial<T>` | All properties optional |
| `Required<T>` | All properties required |
| `Readonly<T>` | All properties readonly |
| `Pick<T, K>` | Keep only keys `K` |
| `Omit<T, K>` | Drop keys `K` |
| `Record<K, V>` | Object type with keys `K` and values `V` |
| `Extract<T, U>` | Union members of `T` assignable to `U` |
| `Exclude<T, U>` | Union members of `T` not assignable to `U` |
| `ReturnType<F>` | Return type of function `F` |
| `Parameters<F>` | Tuple of a function's parameter types |
| `Awaited<T>` | Unwraps nested `Promise<...>` |
| `NonNullable<T>` | Removes `null`/`undefined` from `T` |

## Composing Utilities

Utility types combine well, avoiding a separate named type for every intermediate shape:

```typescript
type UserSummary = Pick<User, "id" | "name">;
type PartialUserSummary = Partial<UserSummary>;

// Extracting a function's async return value
async function fetchUser(): Promise<User> {
  /* ... */
}
type FetchedUser = Awaited<ReturnType<typeof fetchUser>>; // User
```

For anything more elaborate (deep partials, recursive readonly, path-based updates), reach for the `type-fest` package rather than hand-rolling — it covers dozens of these well-tested patterns (`PartialDeep`, `ReadonlyDeep`, `SetOptional`, etc.).

## See Also

- [type-satisfies-operator](type-satisfies-operator.md) - Use satisfies to validate a value's shape without widening its type
- [api-interface-vs-type](api-interface-vs-type.md) - Choosing between interface and type declarations
- [imm-deep-immutability-types](imm-deep-immutability-types.md) - Modeling deep immutability at the type level
