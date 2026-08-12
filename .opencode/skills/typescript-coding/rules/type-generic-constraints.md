# type-generic-constraints

> Constrain generic type parameters with `extends` instead of leaving them unbounded

## Why It Matters

An unconstrained generic parameter (`function foo<T>(x: T)`) tells the compiler nothing about what operations are valid on `T`, so the function body is forced to treat it as opaque — no property access, no method calls, nothing useful. Worse, callers get no feedback if they pass something structurally incompatible with what the function actually needs, because "anything" is technically valid. Adding an `extends` bound documents the real requirement, unlocks safe property/method access inside the function, and gives callers a precise compile error when their argument doesn't fit.

## Bad

```typescript
function getLength<T>(value: T): number {
  return value.length; // Error: Property 'length' does not exist on type 'T'
}

function mergeConfigs<T>(base: T, override: T): T {
  return { ...base, ...override }; // works, but T could be a number or string here
}

mergeConfigs(5, 10); // compiles and returns {} — nonsensical for primitives
```

## Good

```typescript
function getLength<T extends { length: number }>(value: T): number {
  return value.length; // OK, T is guaranteed to have `length`
}

getLength("hello"); // OK
getLength([1, 2, 3]); // OK
getLength(5); // Error: number doesn't satisfy { length: number }

function mergeConfigs<T extends Record<string, unknown>>(base: T, override: Partial<T>): T {
  return { ...base, ...override };
}

mergeConfigs({ port: 3000 }, { port: 4000 }); // OK
mergeConfigs(5, 10); // Error: number doesn't satisfy Record<string, unknown>
```

## Constraining With `keyof` for Safe Property Access

```typescript
function getProp<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

const user = { id: 1, name: "Ada" };
getProp(user, "name"); // OK, inferred as string
getProp(user, "email"); // Error: "email" is not assignable to "id" | "name"
```

## Common Constraint Patterns

| Constraint | Use case |
|---|---|
| `T extends object` | Reject primitives, allow any object shape |
| `T extends Record<string, unknown>` | Plain-object-like structures (config, DTOs) |
| `T extends { id: string \| number }` | Entities that must be identifiable |
| `T extends unknown[]` | Array-specific generic helpers |
| `T extends (...args: never[]) => unknown` | Function-specific generic helpers |

## See Also

- [type-utility-types](type-utility-types.md) - Prefer built-in utility types over hand-rolled equivalents
- [api-generic-defaults](api-generic-defaults.md) - Providing sensible default type parameters for generics
- [name-generic-type-params](name-generic-type-params.md) - Naming conventions for generic type parameters
