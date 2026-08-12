# doc-deprecated-tag

> Use `@deprecated` with migration guidance when retiring an API

## Why It Matters

API consumers need time to migrate away from deprecated functions. The `@deprecated` tag makes the deprecation visible in IDEs (usually with a strikethrough), and the accompanying message tells consumers what to use instead. Without migration guidance, consumers are left guessing what replaced the old API.

## Bad

```js
// Deprecated without documentation — consumers are surprised
export function oldFormat(data) {
  return formatV2(data);
}

// Vague deprecation
/** @deprecated */
export function legacyProcess(data) {
  return newProcess(data);
}
```

## Good

```js
/**
 * Formats data using the legacy schema.
 *
 * @param {Object} data - The raw data
 * @returns {Object} Formatted data
 * @deprecated Since v3.0.0. Use {@link formatV2} instead.
 * Will be removed in v4.0.0.
 *
 * @example
 * ```js
 * // Old (deprecated)
 * const result = oldFormat(data);
 *
 * // New (preferred)
 * const result = formatV2(data, { schema: '2024' });
 * ```
 */
export function oldFormat(data) {
  console.warn('oldFormat is deprecated. Use formatV2 instead.');
  return formatV2(data);
}
```

## Deprecation Warnings

```js
/**
 * @deprecated Use {@link createInvoice} with `options.template` instead.
 */
export function createInvoiceLegacy(order) {
  if (process.env.NODE_ENV !== 'production') {
    console.warn(new Error(
      'createInvoiceLegacy is deprecated. Use createInvoice with options.template.'
    ));
  }
  return createInvoice(order, { template: 'legacy' });
}

// In production, log once
let deprecationWarned = false;

export function oldMethod() {
  if (!deprecationWarned) {
    console.warn('oldMethod is deprecated. Use newMethod.');
    deprecationWarned = true;
  }
  return newMethod();
}
```

## When Exceptions Apply

Internal functions that aren't part of the public API can be removed directly without deprecation. The `@deprecated` tag is for functions that consumers import.

## See Also

- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
- [doc-see-references](./doc-see-references.md) - Cross-references
