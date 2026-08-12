# name-generic-type-params

> Use conventional short generic names (`T`, `K`, `V`, `E`) or a descriptive name for complex generics

## Why It Matters

Single-letter generic names (`T`, `K`, `V`, `E`, `R`) follow a well-established convention that experienced TypeScript readers recognize instantly — `T` for a generic "type," `K`/`V` for map key/value, `E` for error/element, `R` for a return type. That convention breaks down once a generic function or type has multiple type parameters whose relationship or meaning isn't obvious from position alone; at that point, a single letter forces the reader to trace usage sites to figure out what each parameter represents, and a descriptive name is unambiguously better.

## Bad

```typescript
// Single generic, clear from context — fine, but the opposite problem shows up below
function identity<T>(value: T): T {
  return value;
}

// Multiple single-letter generics with non-obvious roles — which is which?
function mergeConfigs<T, U, V>(base: T, override: U, fallback: V): T & U & V {
  return { ...fallback, ...base, ...override };
}

interface Repository<T, K, F> {
  findById(id: K): T | undefined;
  findMany(filter: F): T[];
}
```

## Good

```typescript
// Single, unambiguous generic — conventional T is perfectly clear
function identity<T>(value: T): T {
  return value;
}

// Multiple generics with distinct roles — descriptive names remove the guesswork
function mergeConfigs<TBase, TOverride, TFallback>(
  base: TBase,
  override: TOverride,
  fallback: TFallback,
): TBase & TOverride & TFallback {
  return { ...fallback, ...base, ...override };
}

interface Repository<TEntity, TId, TFilter> {
  findById(id: TId): TEntity | undefined;
  findMany(filter: TFilter): TEntity[];
}
```

## Conventional Single-Letter Names

| Letter | Conventional meaning |
|---|---|
| `T` | A generic type (the default, when there's only one) |
| `K` | A key type, typically constrained to `keyof` something |
| `V` | A value type, paired with `K` (map/record value) |
| `E` | An element type (array/collection) or an error type |
| `R` | A return/result type |
| `P` | Props (common in React/JSX generic components) |

## Rule Of Thumb

- One or two generics with an obvious, conventional role (`T`, `K`/`V` on a map-like structure): single letters are clearer, not less clear, because they match reader expectations.
- Three or more generics, or generics whose relationship to each other matters (a base type vs. an override type vs. a filter type): prefix with `T` and a descriptive word (`TEntity`, `TFilter`) so each one is self-explanatory without cross-referencing usage.

## See Also

- [name-PascalCase-types](name-PascalCase-types.md) - Use `PascalCase` for types, interfaces, classes, and enums
- [type-generic-constraints](type-generic-constraints.md) - Constrain generic type parameters instead of leaving them unbounded
- [api-generic-defaults](api-generic-defaults.md) - Provide sensible default type arguments for generics
