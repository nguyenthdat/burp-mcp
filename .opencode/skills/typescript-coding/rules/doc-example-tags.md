# doc-example-tags

> Include an `@example` block in non-trivial doc comments

## Why It Matters

A prose description tells a consumer what a function does in the abstract, but an `@example` block shows exactly how to call it — the argument shapes, the return value, and common usage idioms — which is what most developers actually copy when integrating a new API. Functions with generics, multiple overloads, or option objects are especially easy to misuse without a worked example, and a wrong guess often only surfaces as a runtime bug rather than a type error.

## Bad

```typescript
/**
 * Retries an async operation with exponential backoff.
 * @param fn - The operation to retry.
 * @param options - Retry configuration.
 */
export function retry<T>(fn: () => Promise<T>, options: RetryOptions): Promise<T> {
  // ...
}
```

## Good

```typescript
/**
 * Retries an async operation with exponential backoff.
 *
 * @param fn - The operation to retry.
 * @param options - Retry configuration.
 *
 * @example
 * Retry a flaky network call up to 5 times, doubling the delay each time:
 * ```typescript
 * const data = await retry(
 *   () => fetch("/api/data").then((r) => r.json()),
 *   { maxAttempts: 5, baseDelayMs: 100 },
 * );
 * ```
 */
export function retry<T>(fn: () => Promise<T>, options: RetryOptions): Promise<T> {
  // ...
}
```

## Guidelines

- Reserve `@example` for functions where the call shape isn't obvious from the signature alone — a trivial `add(a: number, b: number): number` doesn't need one.
- Show the *realistic* usage, not a synthetic minimal call — prefer an example that mirrors how the function is actually used in the codebase.
- For functions with multiple common usage modes (e.g. a builder with several configuration paths), include one `@example` block per mode rather than cramming all variants into a single snippet.
- Keep examples runnable and in sync: if the signature changes, the example must change with it in the same PR — a stale example is worse than none, because it teaches the wrong API.
- `typedoc` and most editor tooling render fenced code blocks inside `@example` with syntax highlighting, so always use a ```typescript fence inside the tag.

## See Also

- [doc-tsdoc-public-api](doc-tsdoc-public-api.md) - Document all public API with TSDoc comments
- [doc-param-returns-tags](doc-param-returns-tags.md) - Document `@param`/`@returns` for signatures that aren't self-evident
- [doc-readme-package](doc-readme-package.md) - Maintain a README with install/usage examples for every published package
