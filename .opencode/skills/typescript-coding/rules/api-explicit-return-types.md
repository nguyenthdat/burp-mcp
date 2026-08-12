# api-explicit-return-types

> Annotate explicit return types on exported functions

## Why It Matters

TypeScript can infer a function's return type from its body, but for exported functions that inference becomes part of your public contract whether you intended it or not — a seemingly unrelated change deep inside the function body can silently widen or narrow the inferred return type and break every consumer, with the error surfacing far away at the call site instead of at the function definition. An explicit return type turns that class of bug into a local compile error exactly where the incompatible change was made, and it also makes the function's contract readable without needing to trace through its implementation.

## Bad

```typescript
// Return type is inferred as whatever the body happens to produce today
export function parseUserId(input: string) {
  if (!input) return null;
  const id = Number(input);
  return Number.isNaN(id) ? null : id;
}

// Later, someone "simplifies" the function and the inferred type
// silently changes from `number | null` to `number | null | undefined`
// (e.g. by adding an early `return;`), and every consumer that
// pattern-matches on exactly `null` now has a latent bug — with no
// error at the point of the change, only wherever `parseUserId` is used.
```

## Good

```typescript
export function parseUserId(input: string): number | null {
  if (!input) return null;
  const id = Number(input);
  return Number.isNaN(id) ? null : id;
}

// If the body's return type ever stops matching `number | null`,
// TypeScript raises the error right here, at the function definition.
```

## Enforcing With ESLint

```jsonc
{
  "rules": {
    "@typescript-eslint/explicit-module-boundary-types": "error"
  }
}
```

This rule specifically targets *exported* functions and class methods — it deliberately does not require annotations on private/internal functions, where inference is safe and annotations are often just noise.

## Where Inference Is Fine (Don't Over-Annotate)

```typescript
// Internal, unexported helper — inference is safe and the annotation
// would just repeat what's already obvious from the body.
function double(n: number) {
  return n * 2;
}

// Arrow functions passed inline as callback arguments — the callback's
// expected type already constrains the return type.
items.map((item) => item.id);
```

## See Also

- [lint-strict-tsconfig](lint-strict-tsconfig.md) - Enabling strict compiler options that catch related inference gaps
- [api-module-boundary-types](api-module-boundary-types.md) - Define explicit DTOs at module/service boundaries, separate from internal domain models
- [api-minimal-surface](api-minimal-surface.md) - Keep the public API surface as small as the consumer actually needs
