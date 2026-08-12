# anti-type-any-return

> Don't return `any` from a function; it erases type safety for every caller

## Why It Matters

A function's return type is the contract every caller relies on — if it's `any`, that contract is void, and the loss of safety propagates outward from a single declaration to every place the function is ever called, including call sites that have no idea the underlying implementation is untyped. This is worse than an `any` parameter (which affects one call site) because a widely-used function with an `any` return type silently disables checking across the entire codebase, one call at a time, often invisibly — the caller's own code looks fully typed, but every value derived from the call is actually unchecked. It's especially easy to introduce accidentally: a function with no explicit return type annotation, calling a poorly-typed third-party API, will have its return type *inferred* as `any` without any visible signal in the function's own source.

## Bad

```typescript
// No explicit return type; TypeScript infers `any` because JSON.parse returns `any`
function loadSettings(raw: string) {
  return JSON.parse(raw);
}

const settings = loadSettings(rawConfig);
settings.theme.mode.toUpperCase(); // no error, even if this path doesn't exist
```

```typescript
// Explicit `any` is even more clearly a problem, but has the same effect
function fetchLegacyData(): any {
  return legacySdk.getData();
}
```

## Good

```typescript
interface Settings {
  theme: { mode: 'light' | 'dark' };
  language: string;
}

function loadSettings(raw: string): Settings {
  return settingsSchema.parse(JSON.parse(raw)); // validated AND typed
}

const settings = loadSettings(rawConfig);
settings.theme.mode.toUpperCase(); // compiler catches a wrong path here
```

```typescript
// For a genuinely untyped third-party SDK, return `unknown` and let
// the caller narrow — don't silently claim a specific shape you can't verify
function fetchLegacyData(): unknown {
  return legacySdk.getData();
}

const data = fetchLegacyData();
if (isValidLegacyShape(data)) {
  process(data); // now safely narrowed
}
```

## Guard Against Accidental Inference

Enable `@typescript-eslint/explicit-function-return-type` (or at minimum `explicit-module-boundary-types`) so every exported function must declare its return type explicitly — this turns an accidental `any` inference into a visible, reviewable annotation instead of a silent gap.

```javascript
// eslint.config.js
export default tseslint.config({
  rules: {
    '@typescript-eslint/explicit-module-boundary-types': 'error',
  },
});
```

## See Also

- [anti-any-abuse](anti-any-abuse.md) - Don't use `any` to silence type errors
- [lint-no-explicit-any](lint-no-explicit-any.md) - Enable `@typescript-eslint/no-explicit-any`
- [api-explicit-return-types](api-explicit-return-types.md) - Declare explicit return types on exported functions
- [type-unknown-over-any](type-unknown-over-any.md) - Prefer `unknown` over `any` for values of uncertain type
