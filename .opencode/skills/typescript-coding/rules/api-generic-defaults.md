# api-generic-defaults

> Give generic type parameters sensible defaults where one exists

## Why It Matters

Without a default, every consumer of a generic type or function must supply a type argument even in the overwhelmingly common case, adding boilerplate and obscuring the "normal" usage under type-parameter noise. A well-chosen default lets the common case read as plain, unparameterized code while still allowing advanced consumers to override it when they genuinely need something different — this is the same instinct as a function parameter default, applied to types.

## Bad

```typescript
interface ApiResponse<T> {
  data: T;
  meta: { requestId: string; timestamp: number };
}

// Every single call site that doesn't care about a specific payload
// shape is still forced to specify one.
function unknownResponse(): ApiResponse<unknown> {
  return { data: undefined, meta: { requestId: "", timestamp: 0 } };
}
```

## Good

```typescript
interface ApiResponse<T = unknown> {
  data: T;
  meta: { requestId: string; timestamp: number };
}

// The common/unspecified case needs no type argument at all:
function genericResponse(): ApiResponse {
  return { data: undefined, meta: { requestId: "", timestamp: 0 } };
}

// Advanced consumers still override it when they need to:
function userResponse(): ApiResponse<User> {
  return { data: fetchedUser, meta: { requestId: "abc", timestamp: Date.now() } };
}
```

## Defaults That Reference Earlier Type Parameters

```typescript
// A default can depend on a preceding type parameter.
interface Repository<TEntity, TKey = TEntity extends { id: infer K } ? K : string> {
  findById(id: TKey): Promise<TEntity | undefined>;
  save(entity: TEntity): Promise<void>;
}

interface Product {
  id: number;
  name: string;
}

// TKey is inferred as `number` here, no need to specify it explicitly
declare const products: Repository<Product>;
```

## Choosing the Right Default

| Situation | Good default |
|---|---|
| Generic container with no natural "empty" payload type | `unknown` (forces callers to narrow before use — safer than `any`) |
| Generic utility that's commonly used with strings | `string` if that's overwhelmingly the common case |
| Generic that mirrors another parameter (e.g. a key type derived from an entity type) | A conditional type default deriving it, as above |

Avoid defaulting to `any` — it silently disables type checking for every consumer who doesn't override the parameter, which defeats the purpose of the generic in the first place.

## See Also

- [type-generic-constraints](type-generic-constraints.md) - Constraining generic type parameters with `extends`
- [type-unknown-over-any](type-unknown-over-any.md) - Preferring `unknown` over `any` as a safe default
- [api-function-overload-order](api-function-overload-order.md) - Order overload signatures from most specific to most general
