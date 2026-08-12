# type-unknown-over-any

> Use `unknown` instead of `any` for values of uncertain type

## Why It Matters

`any` disables the type checker entirely, silently propagating through every operation you perform on it, so a typo or a wrong argument type will only surface as a runtime crash. `unknown` still lets you accept a value of uncertain shape, but the compiler forces you to narrow it (via a type guard, `typeof`, `instanceof`, or a schema parse) before you can do anything with it. Switching `any` to `unknown` at API boundaries turns a whole class of latent bugs into compile-time errors, without losing the flexibility of accepting arbitrary input.

## Bad

```typescript
function parseConfig(raw: any) {
  // No error here, even though raw might not have `port` at all
  const port = raw.port.toFixed(2);
  return { port };
}

parseConfig({ port: "8080" }); // compiles, throws at runtime: toFixed is not a function
parseConfig(null); // compiles, throws at runtime: Cannot read properties of null
```

## Good

```typescript
function parseConfig(raw: unknown) {
  if (
    typeof raw !== "object" ||
    raw === null ||
    !("port" in raw) ||
    typeof raw.port !== "number"
  ) {
    throw new Error("invalid config: expected { port: number }");
  }
  // raw is narrowed to { port: number } here
  const port = raw.port.toFixed(2);
  return { port };
}

parseConfig({ port: 8080 }); // OK
parseConfig({ port: "8080" }); // throws a clear, intentional error
```

## Where `any` Still Sneaks In

| Source | Mitigation |
|---|---|
| Untyped third-party libraries | Write a local `.d.ts` or narrow immediately at the import boundary |
| `JSON.parse()` return type | Always `unknown`, then validate with a schema (see `type-zod-schema-inference`) |
| Legacy code migration | Use `unknown` plus `// TODO(narrow)` markers instead of blanket `any` |
| Catch clause bindings | Already `unknown` by default under `useUnknownInCatchVariables` (TS 4.4+) |

## Configuration

Ban implicit and explicit `any` at the linter level so regressions are caught in CI:

```json
{
  "rules": {
    "@typescript-eslint/no-explicit-any": "error"
  }
}
```

## See Also

- [type-narrow-guards](type-narrow-guards.md) - Use user-defined type guards to narrow union types safely
- [type-zod-schema-inference](type-zod-schema-inference.md) - Derive static types from a runtime schema instead of maintaining both by hand
- [lint-no-explicit-any](lint-no-explicit-any.md) - Lint rule banning explicit any usage
- [err-typed-catch-unknown](err-typed-catch-unknown.md) - Type the catch binding as unknown and narrow before use
