# lint-no-explicit-any

> Enable `@typescript-eslint/no-explicit-any`

## Why It Matters

`any` disables type-checking for a value and, worse, for everything derived from it — a single `any` at a function boundary silently propagates through every call site that touches the return value, so a typo'd property access or a wrong argument type produces no compiler error anywhere downstream. Because `any` is easy to reach for under deadline pressure ("I'll type this properly later"), it needs a lint rule rather than a style guideline; `@typescript-eslint/no-explicit-any` turns every occurrence into a visible, reviewable diagnostic instead of a silent hole in the type system.

## Bad

```typescript
// eslint.config.js — rule not enabled, so this passes lint silently
function parseResponse(data: any) {
  return data.items.map((item: any) => item.value.toUpperCase());
  // any typo here (e.g. item.vaule) is not caught until runtime
}
```

## Good

```javascript
// eslint.config.js
import tseslint from 'typescript-eslint';

export default tseslint.config({
  rules: {
    '@typescript-eslint/no-explicit-any': 'error',
  },
});
```

```typescript
interface ApiResponse {
  items: Array<{ value: string }>;
}

function parseResponse(data: ApiResponse) {
  return data.items.map((item) => item.value.toUpperCase());
  // item.vaule now fails to compile
}

// If the shape is genuinely unknown ahead of time, use `unknown` and narrow it
function parseUnknown(data: unknown) {
  if (isApiResponse(data)) {
    return data.items.map((item) => item.value.toUpperCase());
  }
  throw new Error('unexpected response shape');
}
```

## Escape Hatch, Used Deliberately

```typescript
// Rare, justified use: interop with an untyped third-party callback
// eslint-disable-next-line @typescript-eslint/no-explicit-any -- vendor SDK has no types
function legacyCallback(err: any, result: any) { ... }
```

Requiring an inline disable comment (rather than a blanket rule-off) keeps every remaining `any` visible in code review and greppable (`no-explicit-any --`) across the codebase, instead of silently tolerated everywhere.

## See Also

- [anti-any-abuse](anti-any-abuse.md) - Don't use `any` to silence type errors
- [type-unknown-over-any](type-unknown-over-any.md) - Prefer `unknown` over `any` for values of uncertain type
- [anti-type-any-return](anti-type-any-return.md) - Don't return `any` from a function; it erases type safety for every caller
