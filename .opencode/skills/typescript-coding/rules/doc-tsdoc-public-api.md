# doc-tsdoc-public-api

> Document all public API with TSDoc comments

## Why It Matters

Without TSDoc comments, consumers of a library or internal package have to read the implementation (or guess from the name) to understand what a function does, what its parameters mean, and what it returns — this slows every integration and leads to misuse of edge cases that aren't obvious from the type signature alone. TSDoc comments are picked up by editor hovers, `typedoc`, and API-extraction tools, turning the type signature plus doc comment into a self-contained contract that IDEs surface exactly where a consumer needs it: at the call site.

## Bad

```typescript
// No documentation — consumers must open the implementation to learn
// what "strict" does, what units the timeout is in, or what's returned.
export function parseConfig(input: string, strict: boolean, timeoutMs: number): Config {
  // ...
}
```

## Good

```typescript
/**
 * Parses a raw configuration string into a validated {@link Config} object.
 *
 * @remarks
 * When `strict` is true, unknown keys cause a {@link ConfigParseError} instead
 * of being silently dropped. Use strict mode in CI and non-strict mode for
 * user-facing config editors where partial/experimental keys are common.
 *
 * @param input - Raw configuration source, typically the contents of `config.json`.
 * @param strict - Whether to reject unknown top-level keys.
 * @param timeoutMs - Maximum time, in milliseconds, to spend resolving `$extends` references.
 * @returns The parsed and validated configuration.
 * @throws {@link ConfigParseError} if `input` is not valid JSON or fails schema validation.
 */
export function parseConfig(input: string, strict: boolean, timeoutMs: number): Config {
  // ...
}
```

## What Counts as "Public"

- Every exported function, class, interface, and type alias from a package's entry point (`index.ts` / whatever `package.json#exports` points to).
- Public class members (public methods, public fields) — private/protected members are documented only when their behavior is non-obvious.
- Internal-only exports used across files in the same package but never published are lower priority; document them with plain comments instead if TSDoc feels excessive.

Run `typedoc --entryPoints src/index.ts` (or wire it into CI) to catch exported symbols missing documentation before they ship.

## See Also

- [doc-example-tags](doc-example-tags.md) - Include an `@example` block in non-trivial doc comments
- [doc-param-returns-tags](doc-param-returns-tags.md) - Document `@param`/`@returns` for signatures that aren't self-evident
- [doc-throws-tag](doc-throws-tag.md) - Document the errors a function can throw with `@throws`
- [api-minimal-surface](api-minimal-surface.md) - keep the public API small enough to document well
