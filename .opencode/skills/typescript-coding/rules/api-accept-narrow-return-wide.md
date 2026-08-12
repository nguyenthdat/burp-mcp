# api-accept-narrow-return-wide

> Accept the most general input types callers already have; return the most specific types

## Why It Matters

This is the "robustness principle" applied to TypeScript signatures: if a parameter only needs to be read, accepting the widest reasonable type (`readonly T[]` instead of `T[]`, `Iterable<T>` instead of `T[]`, a structural interface instead of a concrete class) lets callers pass whatever they already have without needless conversion. Conversely, returning the most specific type you can (a concrete union, a literal type, a branded type) gives callers maximum information to work with instead of forcing them to widen/narrow it themselves. Getting this backwards — accepting an overly specific type, or returning an overly general one — pushes unnecessary type gymnastics onto every caller.

## Bad

```typescript
// Accepts a concrete, mutable array — forces callers who have a
// readonly array or any other iterable to first copy it into one.
function sum(values: number[]): number {
  return values.reduce((a, b) => a + b, 0);
}

// Returns a wide `string` — callers who need to know it's specifically
// one of a fixed set of statuses have to re-validate/re-narrow it.
function getStatus(): string {
  return Math.random() > 0.5 ? "active" : "inactive";
}
```

## Good

```typescript
// Accepts anything iterable — arrays, Sets, readonly arrays, generators
function sum(values: Iterable<number>): number {
  let total = 0;
  for (const v of values) total += v;
  return total;
}

sum([1, 2, 3]);                 // array
sum(new Set([1, 2, 3]));        // Set
sum(readonlyNumbers);           // readonly array — no copy needed

// Returns the specific literal union — callers get full type information
function getStatus(): "active" | "inactive" {
  return Math.random() > 0.5 ? "active" : "inactive";
}
```

## Common Widening Opportunities for Parameters

| Instead of | Accept |
|---|---|
| `T[]` (when not mutating) | `readonly T[]` or `ReadonlyArray<T>` |
| A concrete class | A structural interface describing only the members you use |
| `Map<K, V>` (when only reading) | `ReadonlyMap<K, V>` |
| `string` (when only a few values are valid) | A literal union, still widened to `string` only if truly open-ended |

## Common Narrowing Opportunities for Return Types

| Instead of | Return |
|---|---|
| `string` for a fixed set of outcomes | A literal union |
| `object` / `Record<string, unknown>` | A specific interface or discriminated union |
| `any` from a third-party call | The actual shape, validated/asserted once at the boundary |

## See Also

- [type-readonly-arrays](type-readonly-arrays.md) - Preferring `readonly T[]` over `T[]` in function signatures
- [api-readonly-public-types](api-readonly-public-types.md) - Mark public interface properties `readonly` unless mutation is part of the contract
- [api-module-boundary-types](api-module-boundary-types.md) - Define explicit DTOs at module/service boundaries, separate from internal domain models
