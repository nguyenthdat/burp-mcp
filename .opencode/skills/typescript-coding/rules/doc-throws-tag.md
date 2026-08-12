# doc-throws-tag

> Document the errors a function can throw with `@throws`

## Why It Matters

TypeScript's type system does not track thrown exceptions in a function's signature, so a caller has no way to know a function can throw at all — let alone what error types — without reading its implementation or every function it transitively calls. Without an explicit `@throws` tag, error handling becomes guesswork: callers either wrap everything in an overly broad `try/catch` "just in case," or skip error handling entirely and let an unanticipated exception crash the process. Documenting `@throws` makes the failure modes part of the function's public contract.

## Bad

```typescript
/**
 * Loads and parses a user's saved preferences.
 */
export async function loadPreferences(userId: string): Promise<Preferences> {
  const raw = await fs.readFile(preferencesPath(userId), "utf8");
  return PreferencesSchema.parse(JSON.parse(raw));
}
```

## Good

```typescript
/**
 * Loads and parses a user's saved preferences from disk.
 *
 * @param userId - The user whose preferences to load.
 * @returns The parsed and validated preferences.
 * @throws {@link NodeJS.ErrnoException} with code `ENOENT` if no preferences
 * file exists for this user yet — callers should catch this and fall back
 * to {@link defaultPreferences}.
 * @throws {@link ZodError} if the stored file is corrupted or was written by
 * an incompatible older schema version.
 */
export async function loadPreferences(userId: string): Promise<Preferences> {
  const raw = await fs.readFile(preferencesPath(userId), "utf8");
  return PreferencesSchema.parse(JSON.parse(raw));
}
```

## Guidelines

- Document every error type a caller might reasonably need to catch and handle differently — not every internal error a function could theoretically throw.
- For custom error classes, link to the class definition with `{@link CustomError}` so the reader can see its fields (e.g. an error `code` used in a `switch`).
- If a function wraps a lower-level call's errors in a domain-specific error (see `err-rethrow-context`), document the wrapped type, not the internal one — that's the contract callers actually see.
- Async functions that return a rejected promise follow the same convention: `@throws` describes rejection reasons, since `Promise<T>` rejections are just as undocumented by the type system as synchronous throws.
- For functions using a `Result`-style return instead of throwing (see `err-result-pattern`), skip `@throws` and document the error variant in `@returns` instead — don't mix both error-handling styles in a single function's docs.

## See Also

- [err-custom-error-class](err-custom-error-class.md) - define typed custom error classes that `@throws` can reference
- [err-result-pattern](err-result-pattern.md) - alternative to throwing: return errors as values
- [doc-tsdoc-public-api](doc-tsdoc-public-api.md) - Document all public API with TSDoc comments
