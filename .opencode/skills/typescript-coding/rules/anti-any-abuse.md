# anti-any-abuse

> Don't use `any` to silence type errors

## Why It Matters

`any` isn't "I don't know the type yet" — it's "turn off type-checking for this value and everything derived from it." Reaching for `any` to make a red squiggle disappear trades a five-second compile error for an unbounded, silent runtime failure surface: every property access, every argument passed, every return value touched by that `any` skips checking, and the loss compounds as the value flows through the rest of the program. It's almost always cheaper to spend the extra minute writing the real type, or to use `unknown` and narrow, than to defer the cost to whoever eventually hits the resulting runtime error in production.

## Bad

```typescript
function processPayload(payload: any) {
  // No error here no matter what payload actually looks like
  return payload.user.profile.settings.theme;
}

async function fetchJson(url: string): Promise<any> {
  const res = await fetch(url);
  return res.json(); // callers get zero type safety on the result
}
```

## Good

```typescript
interface Payload {
  user: {
    profile: {
      settings: { theme: string };
    };
  };
}

function processPayload(payload: Payload) {
  return payload.user.profile.settings.theme; // typo'd path fails to compile
}

async function fetchJson<T>(url: string, schema: z.ZodType<T>): Promise<T> {
  const res = await fetch(url);
  return schema.parse(await res.json()); // validated AND typed
}
```

## Decision Guide

| Situation | Use |
|---|---|
| Shape is genuinely unknown until runtime (parsed JSON, external API) | `unknown`, then narrow or validate with a schema |
| Shape is known but tedious to write out | Write it anyway, or generate it (OpenAPI codegen, Zod inference) |
| Interop with an untyped third-party library | A scoped, documented type assertion or a minimal ambient declaration — not `any` everywhere it's used |
| "I'll fix the type later" | Don't; that "later" rarely comes and the `any` spreads |

## See Also

- [lint-no-explicit-any](lint-no-explicit-any.md) - Enable `@typescript-eslint/no-explicit-any`
- [type-unknown-over-any](type-unknown-over-any.md) - Prefer `unknown` over `any` for values of uncertain type
- [anti-type-any-return](anti-type-any-return.md) - Don't return `any` from a function; it erases type safety for every caller
- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from Zod schemas instead of duplicating them
