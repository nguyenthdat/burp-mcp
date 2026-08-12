# doc-private-internal

> Use `@private` and `@internal` JSDoc tags to mark non-public APIs

## Why It Matters

Not every exported function is part of the public API. Without `@private`/`@internal` tags, documentation generators include internal functions in the public API docs, and consumers may depend on them. These tags signal that a function is exported for technical reasons (testing, internal module access) but is not a supported public API.

## Bad

```js
// Exported for testing but shown as public API
export function _validateEmail(email) {
  return /\S+@\S+/.test(email);
}

// Internal helper shown in public docs
export function formatInternalId(id) {
  return `INT-${id}`;
}
```

## Good

```js
/**
 * Validates an email address format.
 *
 * @param {string} email - The email to validate
 * @returns {boolean} True if the email format is valid
 * @private - Exported for testing only. Do not use directly.
 */
export function _validateEmail(email) {
  return /\S+@\S+/.test(email);
}

/**
 * @private
 * @internal - Subject to change without notice.
 */
export function formatInternalId(id) {
  return `INT-${id}`;
}

// Better: don't export private functions at all
function validateEmail(email) {
  return /\S+@\S+/.test(email);
}

// If you must export for testing, use a test-only export pattern
export const __test__ = process.env.NODE_ENV === 'test'
  ? { validateEmail }
  : undefined;
```

## When Exceptions Apply

In library packages, the `exports` field in `package.json` can physically prevent access to internal modules, making `@private` less critical but still useful for documentation.

## See Also

- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
- [mod-package-exports](./mod-package-exports.md) - Exports field in package.json
