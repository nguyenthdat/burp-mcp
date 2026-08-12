# api-interface-vs-type

> Use `interface` for extendable object shapes, `type` for unions/aliases/mapped types

## Why It Matters

`interface` and `type` overlap for plain object shapes, but they aren't interchangeable everywhere: only `interface` supports declaration merging (multiple declarations combining into one, which libraries rely on to let consumers augment a shape) and produces clearer, more incremental error messages when extended incorrectly. Only `type` can express unions, tuples, conditional types, and mapped types at all — `interface` has no equivalent syntax for those. Picking consistently based on what the shape actually is (an open, extendable object contract vs. an alias/union/computed type) keeps the codebase predictable and avoids reaching for `interface` in places it can't do the job.

## Bad

```typescript
// Using `type` for something meant to be extended/merged by consumers
// of a library — no declaration merging is possible, so this can't be
// augmented the way library consumers typically expect global types to be.
type RequestConfig = {
  url: string;
  method: string;
};

// Using `interface` for a union — this is not valid syntax at all
interface Status {
  // no way to express "pending" | "success" | "error" as an interface
}
```

## Good

```typescript
// interface: an open, extendable object contract
interface RequestConfig {
  url: string;
  method: string;
}

// Consumers (or later versions of the same library) can merge in more:
interface RequestConfig {
  headers?: Record<string, string>;
}

// type: unions, aliases, mapped/conditional types
type Status = "pending" | "success" | "error";
type ReadonlyConfig = Readonly<RequestConfig>;
type ExtractIds<T extends { id: unknown }> = T["id"];
```

## Decision Table

| Shape | Use |
|---|---|
| Object shape that a public API might need to extend/merge (e.g. ambient/global augmentation) | `interface` |
| Union of literal types or object shapes | `type` |
| Tuple type | `type` |
| Mapped or conditional type | `type` |
| Function type alias | `type` (or `interface` with a call signature, less common) |
| Plain internal object shape, no merging needed | Either is fine — pick one convention and apply it consistently (many style guides default to `interface`) |

## `extends` vs Intersection

```typescript
// interface extends — checked structurally at declaration time,
// produces a clear error if the extension is incompatible
interface Admin extends User {
  permissions: string[];
}

// type intersection — more flexible (works with unions too) but can
// silently produce `never` if the intersected members conflict
type Admin = User & { permissions: string[] };
```

## See Also

- [name-interface-no-I-prefix](name-interface-no-I-prefix.md) - Don't prefix interface names with `I`
- [api-avoid-enum-const-object](api-avoid-enum-const-object.md) - Prefer literal unions or `as const` objects over `enum`
- [type-discriminated-union](type-discriminated-union.md) - Modeling variant data with discriminated unions
