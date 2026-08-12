# doc-deprecated-tag

> Mark deprecated APIs with `@deprecated` and a migration path

## Why It Matters

Removing or changing an API without a deprecation period breaks every consumer at once with no warning; marking it `@deprecated` without saying what to use instead just tells developers "this is bad" without telling them how to fix it, so they either ignore the warning or waste time reverse-engineering the replacement. A proper `@deprecated` tag with a concrete migration path lets editors strike through the old call sites, lets `typescript-eslint`'s `deprecation` rule flag new usages in CI, and gives consumers a clear, actionable next step.

## Bad

```typescript
/**
 * @deprecated
 */
export function fetchUserLegacy(id: string): Promise<User> {
  return fetchUser(id);
}
```

## Good

```typescript
/**
 * Fetches a user by id.
 *
 * @deprecated Use {@link fetchUser} instead, which returns a `Result<User, NotFoundError>`
 * rather than throwing. Scheduled for removal in v4.0 (see MIGRATION.md#user-fetch).
 * @param id - The user's id.
 */
export function fetchUserLegacy(id: string): Promise<User> {
  return fetchUser(id);
}
```

## Guidelines

- Always say what to use instead, and link to it with `{@link newFunction}` so editors can jump straight there.
- State the removal timeline if one exists ("removed in v4.0") so consumers can prioritize the migration against other work.
- For larger breaking changes, link to a `MIGRATION.md` section with a worked before/after example rather than cramming the full rationale into the doc comment.
- Enable `@typescript-eslint/no-deprecated` (or the `deprecation` plugin) in CI so new code can't introduce fresh calls to a deprecated API — this is what makes the tag more than a comment nobody reads.
- Keep the deprecated function's behavior working and tested until it's actually removed; a `@deprecated` tag is a promise of continued support during the migration window, not an excuse to let it rot.

## See Also

- [doc-changelog-semver](doc-changelog-semver.md) - Maintain a CHANGELOG that follows semantic versioning
- [api-versioned-public-api](api-versioned-public-api.md) - version a public API deliberately across breaking changes
- [doc-tsdoc-public-api](doc-tsdoc-public-api.md) - Document all public API with TSDoc comments
