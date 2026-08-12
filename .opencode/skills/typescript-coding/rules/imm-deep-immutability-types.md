# imm-deep-immutability-types

> Use a deep-readonly utility type for nested immutable state trees

## Why It Matters

TypeScript's built-in `Readonly<T>` is shallow: it stops reassignment of top-level properties but leaves nested objects and arrays fully mutable, which is a silent trap in state-management code where the whole point is that a state tree never mutates in place. Modeling deeply-nested state (Redux stores, config trees, parsed API responses you want to treat as immutable) with a proper deep-readonly type surfaces accidental nested mutation as a compile error instead of a runtime bug discovered in production.

## Bad

```typescript
interface AppState {
  user: { id: string; profile: { displayName: string; avatarUrl: string } };
  cart: { items: { sku: string; qty: number }[] };
}

function reducer(state: Readonly<AppState>, action: Action): AppState {
  // Readonly<T> only protects the top level:
  state.user.profile.displayName = "hacked"; // compiles! `profile` isn't readonly
  state.cart.items.push({ sku: "X", qty: 1 }); // compiles! array isn't readonly
  return state;
}
```

## Good

```typescript
type DeepReadonly<T> = T extends (infer U)[]
  ? ReadonlyArray<DeepReadonly<U>>
  : T extends Function
    ? T
    : T extends object
      ? { readonly [K in keyof T]: DeepReadonly<T[K]> }
      : T;

interface AppState {
  user: { id: string; profile: { displayName: string; avatarUrl: string } };
  cart: { items: { sku: string; qty: number }[] };
}

function reducer(state: DeepReadonly<AppState>, action: Action): AppState {
  state.user.profile.displayName = "hacked"; // Error: readonly property
  state.cart.items.push({ sku: "X", qty: 1 }); // Error: push does not exist on readonly array

  return {
    ...state,
    cart: { items: [...state.cart.items, { sku: "X", qty: 1 }] },
  };
}
```

## Library Options Instead Of Hand-Rolling

Hand-rolled `DeepReadonly` types are easy to get subtly wrong around tuples, `Map`/`Set`, and function-valued properties. For anything beyond simple state shapes, prefer a maintained implementation:

- **`type-fest`**'s `ReadonlyDeep<T>` — handles `Map`, `Set`, tuples, and class instances correctly.
- **Immer**'s `Immutable<T>` type (paired with `produce`) — matches the runtime immutability Immer already enforces, so types and behavior stay in sync.

```typescript
import type { ReadonlyDeep } from "type-fest";

function selectDisplayName(state: ReadonlyDeep<AppState>): string {
  return state.user.profile.displayName;
}
```

## Pairing Type-Level and Runtime Guarantees

A `DeepReadonly<T>` type only stops mistakes made through code the compiler checks. If the same object can be reached through an `any`, a third-party callback, or deserialized JSON, pair the type with `Object.freeze`/`deepFreeze` at the boundary where the object is created, so the guarantee holds at runtime too.

## See Also

- [imm-object-freeze-runtime](imm-object-freeze-runtime.md) - Use `Object.freeze` when you need a runtime immutability guarantee, not just a compile-time one
- [type-readonly-arrays](type-readonly-arrays.md) - Prefer `ReadonlyArray<T>` for array parameters you don't intend to mutate
- [imm-structural-sharing](imm-structural-sharing.md) - Use structural sharing so immutable updates don't copy untouched subtrees
