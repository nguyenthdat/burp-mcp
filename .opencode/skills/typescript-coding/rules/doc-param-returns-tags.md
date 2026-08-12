# doc-param-returns-tags

> Document `@param`/`@returns` for signatures that aren't self-evident

## Why It Matters

TypeScript's type system already tells a caller *what type* a parameter or return value has; `@param`/`@returns` should explain what the type-checker can't — units, valid ranges, sentinel values, and side effects. Omitting them on non-obvious signatures forces callers to read the implementation to answer basic questions ("is this a timeout in seconds or milliseconds?", "does an empty array mean 'no results' or 'not yet loaded'?"), while adding them on self-evident signatures is noise that makes real documentation easier to skip past.

## Bad

```typescript
/**
 * @param a
 * @param b
 * @returns the result
 */
export function resize(a: number, b: number): { width: number; height: number } {
  // ...
}
```

## Good

```typescript
/**
 * Computes the largest dimensions that fit within a bounding box while
 * preserving the original aspect ratio.
 *
 * @param maxWidth - Maximum allowed width, in pixels.
 * @param maxHeight - Maximum allowed height, in pixels.
 * @returns The scaled `{ width, height }`, both integers, that fit within
 * the bounds without exceeding either dimension.
 */
export function resize(maxWidth: number, maxHeight: number): { width: number; height: number } {
  // ...
}
```

## When To Skip It

Don't add `@param`/`@returns` when the name and type already say everything:

```typescript
/** Adds two numbers. */
export function add(a: number, b: number): number {
  return a + b;
}
```

Reserve the tags for signatures involving units, encodings, nullable/sentinel returns, index bases (0-based vs 1-based), or any parameter whose valid range or format isn't obvious from its type (e.g. `cronExpression: string` needs an example of the expected format).

## Guidelines

- State units explicitly: "in milliseconds", "as a percentage from 0 to 100".
- Call out `undefined`/`null` returns and what they mean: "returns `undefined` if no matching record exists" is much clearer than leaving the reader to infer it from `T | undefined`.
- For destructured parameters, document each field: `@param options.retries - Number of retry attempts before giving up.`
- Keep descriptions imperative and short — a full sentence per tag is usually enough; longer explanations belong in the doc comment's main body or an `@remarks` block.

## See Also

- [doc-tsdoc-public-api](doc-tsdoc-public-api.md) - Document all public API with TSDoc comments
- [doc-example-tags](doc-example-tags.md) - Include an `@example` block in non-trivial doc comments
- [doc-type-as-documentation](doc-type-as-documentation.md) - Let precise types replace comments that only describe a shape
