# type-const-assertion

> Use `as const` to infer literal, readonly types

## Why It Matters

By default, TypeScript widens object and array literals to their general types: a string property becomes `string`, an array becomes `T[]`, and both remain mutable. This loses precision that's often exactly what you want to preserve, for example when a value is used as a discriminant, a tuple, or a fixed set of allowed strings. `as const` tells the compiler to infer the narrowest possible literal type and mark the structure `readonly`, catching accidental mutation and enabling much more precise downstream inference.

## Bad

```typescript
// Widened to { method: string; url: string }
const request = { method: "GET", url: "/users" };

function send(req: { method: "GET" | "POST"; url: string }) {
  // ...
}

send(request); // Error: string is not assignable to "GET" | "POST"

// Widened to number[], and mutable
const directions = [0, 1, 2, 3];
directions.push(4); // compiles, even though this list is meant to be fixed
```

## Good

```typescript
// Inferred as { readonly method: "GET"; readonly url: "/users" }
const request = { method: "GET", url: "/users" } as const;

function send(req: { method: "GET" | "POST"; url: string }) {
  // ...
}

send(request); // OK, "GET" is a valid literal member of "GET" | "POST"

// Inferred as readonly [0, 1, 2, 3] — a fixed-length tuple
const directions = [0, 1, 2, 3] as const;
directions.push(4); // Error: Property 'push' does not exist on type 'readonly [0, 1, 2, 3]'
```

## Deriving Union Types From Values

`as const` combined with indexed access lets you derive a union type from a single source of truth instead of duplicating a literal union declaration:

```typescript
const ROLES = ["admin", "editor", "viewer"] as const;
type Role = (typeof ROLES)[number]; // "admin" | "editor" | "viewer"

function hasRole(role: string): role is Role {
  return (ROLES as readonly string[]).includes(role);
}
```

## `as const` vs Explicit Readonly Types

| Goal | Approach |
|---|---|
| Freeze a single object literal you author inline | `as const` |
| Freeze a parameter type contributed by callers | `readonly T[]` / `Readonly<T>` |
| Derive a union from a fixed list | `as const` + `(typeof arr)[number]` |
| Deep, nested immutability guarantee | `as const` (shallow-safe for literals) or a `DeepReadonly<T>` utility |

Note that `as const` only affects the literal it's applied to — it does not recursively freeze values that were already computed elsewhere and merely referenced.

## See Also

- [type-satisfies-operator](type-satisfies-operator.md) - Use satisfies to validate a value's shape without widening its type
- [type-readonly-arrays](type-readonly-arrays.md) - Accept readonly T[] for parameters that shouldn't be mutated
- [imm-as-const-literal](imm-as-const-literal.md) - Using as const for immutable literal values
