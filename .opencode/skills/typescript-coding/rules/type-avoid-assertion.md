# type-avoid-assertion

> Avoid `as` type assertions; prefer narrowing or validation

## Why It Matters

A type assertion (`value as Type`) tells the compiler "trust me," without performing any runtime check — if you're wrong, the mistake compiles cleanly and surfaces later as a confusing runtime error far from where it originated. Every assertion is a spot where the type checker's guarantees have a hole poked in them. Preferring narrowing (type guards, `typeof`/`instanceof`, discriminated unions) or actual validation (schema parsing) keeps the compiler's safety net intact and makes bad data fail loudly, close to its source.

## Bad

```typescript
function getUser(id: string): User {
  const cached = cache.get(id);
  return cached as User; // lies if cache.get returns undefined or a stale shape
}

async function handleRequest(req: Request) {
  const body = (await req.json()) as { userId: string; amount: number };
  // If the real payload doesn't match, this silently produces `undefined` fields
  await charge(body.userId, body.amount);
}
```

## Good

```typescript
function getUser(id: string): User | undefined {
  const cached = cache.get(id);
  if (!isUser(cached)) {
    return undefined;
  }
  return cached; // narrowed by a real type guard, not asserted
}

function isUser(value: unknown): value is User {
  return (
    typeof value === "object" &&
    value !== null &&
    "id" in value &&
    "name" in value
  );
}

async function handleRequest(req: Request) {
  const raw: unknown = await req.json();
  const body = ChargeRequestSchema.parse(raw); // throws on mismatch, types are derived
  await charge(body.userId, body.amount);
}
```

## When an Assertion Is Legitimate

Assertions aren't universally forbidden — they're appropriate when you have information the compiler structurally can't infer, and you're prepared to own the risk:

- Narrowing a `unknown`/generic DOM API result you've already validated by other means (e.g. `document.getElementById("app")!` after confirming the element always exists in the markup).
- Test code asserting a mock/fixture shape that's known to be correct.
- Interop with libraries whose types are wrong or missing (prefer fixing the `.d.ts` instead, if feasible).

Even then, prefer a named helper (`asKnownElement`, `assertDefined`) that documents *why* the assertion is safe, over a bare inline `as`.

## Assertion vs Alternatives

| Situation | Prefer |
|---|---|
| Value from `JSON.parse`, `fetch`, external API | Schema validation (zod, valibot) |
| Union narrowing based on a property | Type guard / discriminated union |
| Value known non-null by surrounding logic | `if (x)` check, not `x!` |
| Truly unreachable code path | `never`-typed exhaustiveness helper |

## See Also

- [type-narrow-guards](type-narrow-guards.md) - Use user-defined type guards to narrow union types safely
- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from a runtime schema instead of maintaining both by hand
- [lint-no-non-null-assertion](lint-no-non-null-assertion.md) - Lint rule restricting the non-null assertion operator
- [anti-any-cast-double](anti-any-cast-double.md) - Avoid double-casting through any to bypass type checks
