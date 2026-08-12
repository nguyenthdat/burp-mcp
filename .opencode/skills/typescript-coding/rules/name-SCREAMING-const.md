# name-SCREAMING-const

> Use `SCREAMING_SNAKE_CASE` for true module-level constants

## Why It Matters

`SCREAMING_SNAKE_CASE` is a strong visual signal that a value is a fixed, globally-relevant constant — a configuration ceiling, a magic number given a name, an environment default — and is meant to be referenced, not recomputed or treated as ordinary local state. Applying it indiscriminately to every `const` (including ones that are just locally-scoped, ordinary values that happen to use the `const` keyword) dilutes the signal: if everything is screaming, nothing is. The convention should track "this is a true constant with module-wide significance," not "this happens to use the `const` keyword."

## Bad

```typescript
// Ordinary local values marked as if they were global constants — noisy and misleading
function calculateTotal(items: Item[]) {
  const SUBTOTAL = items.reduce((sum, i) => sum + i.price, 0);
  const TAX_AMOUNT = SUBTOTAL * 0.08;
  return SUBTOTAL + TAX_AMOUNT;
}

// True module-level constants NOT marked, easy to miss their significance
const maxRetries = 3;
const defaultTimeoutMs = 5000;
export const apiBaseUrl = "https://api.example.com";
```

## Good

```typescript
function calculateTotal(items: Item[]) {
  const subtotal = items.reduce((sum, i) => sum + i.price, 0);
  const taxAmount = subtotal * 0.08;
  return subtotal + taxAmount;
}

const MAX_RETRIES = 3;
const DEFAULT_TIMEOUT_MS = 5000;
export const API_BASE_URL = "https://api.example.com";
```

## What Qualifies As A "True Constant"

- Module-level, exported or not, fixed for the lifetime of the program (not derived from runtime input).
- Represents a meaningful limit, default, or fixed identifier: `MAX_UPLOAD_SIZE_BYTES`, `RETRY_BACKOFF_MS`, `SUPPORTED_LOCALES`.
- Primitive or frozen literal values — not object instances with behavior, and not something computed from a function argument.

## What Does Not Qualify

- A `const` inside a function body, even if it's never reassigned — that's just normal `camelCase` local variable naming (see `imm-prefer-const`).
- A frozen configuration *object* — the object itself can be `camelCase` even if you use `SCREAMING_SNAKE_CASE` for the primitive constants nested inside a dedicated constants module.

```typescript
// constants.ts
export const MAX_RETRIES = 3;
export const DEFAULT_TIMEOUT_MS = 5000;

// config.ts — the object itself is camelCase; only true leaf constants scream
export const defaultRequestConfig = {
  retries: MAX_RETRIES,
  timeoutMs: DEFAULT_TIMEOUT_MS,
};
```

## See Also

- [name-camelCase-vars](name-camelCase-vars.md) - Use `camelCase` for variables and functions
- [imm-prefer-const](imm-prefer-const.md) - Default to `const`; use `let` only when a binding is reassigned
- [imm-as-const-literal](imm-as-const-literal.md) - Freeze literal object/array structures with `as const`
