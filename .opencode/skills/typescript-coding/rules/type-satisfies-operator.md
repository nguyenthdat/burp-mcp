# type-satisfies-operator

> Use `satisfies` to validate a value's shape without widening its type

## Why It Matters

Annotating a value with `: SomeType` widens it to exactly that type, throwing away the more specific literal types TypeScript would otherwise infer, and it also loses excess-property checking benefits in some contexts. Casting with `as SomeType` skips validation entirely. The `satisfies` operator (TypeScript 4.9+) checks that a value conforms to a type while preserving the narrower inferred type, so you get both compile-time validation and precise autocompletion/literal types afterward.

## Bad

```typescript
type Config = Record<string, { port: number; host: string }>;

// Annotating widens every value's type to `{ port: number; host: string }`,
// so `route.port` below is just `number`, losing the literal 5432
const config: Config = {
  api: { port: 3000, host: "localhost" },
  db: { port: 5432, host: "localhost" },
};

const dbPort = config.db.port; // type: number (not 5432)

// A typo in a key isn't caught because Config is Record<string, ...>
const config2: Config = {
  ap1: { port: 3000, host: "localhost" }, // typo compiles fine
};
```

## Good

```typescript
type Config = Record<string, { port: number; host: string }>;

// satisfies validates shape but keeps the literal, precise inferred type
const config = {
  api: { port: 3000, host: "localhost" },
  db: { port: 5432, host: "localhost" },
} satisfies Config;

const dbPort = config.db.port; // type: 5432, and `db` autocompletes as a known key

// Combining with `as const` locks values as readonly literals too
const routes = {
  home: "/",
  users: "/users",
} as const satisfies Record<string, `/${string}`>;
```

## satisfies vs Annotation vs Assertion

| Approach | Validates shape | Preserves literal types | Excess property check |
|---|---|---|---|
| `const x: T = value` | Yes | No (widens to `T`) | Yes |
| `const x = value as T` | No | No | No |
| `const x = value satisfies T` | Yes | Yes | Yes |

## Common Pattern: Config Objects and Route Maps

`satisfies` is especially useful for objects consumed by both a strict interface and code that wants the specific literal keys/values, such as route tables, theme tokens, or event-name maps:

```typescript
type EventMap = Record<string, (...args: unknown[]) => void>;

const handlers = {
  onClick: (x: number, y: number) => console.log(x, y),
  onKeyDown: (key: string) => console.log(key),
} satisfies EventMap;

handlers.onClick(1, 2); // still typed as (x: number, y: number) => void, not the generic signature
```

## See Also

- [type-const-assertion](type-const-assertion.md) - Use as const to infer literal, readonly types
- [type-avoid-assertion](type-avoid-assertion.md) - Avoid as type assertions; prefer narrowing or validation
- [type-utility-types](type-utility-types.md) - Prefer built-in utility types over hand-rolled equivalents
