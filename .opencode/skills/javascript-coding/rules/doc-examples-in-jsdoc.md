# doc-examples-in-jsdoc

> Include `@example` blocks for complex or non-obvious functions

## Why It Matters

Reading a function's implementation to understand how to use it is slow and error-prone. An `@example` block shows the intended usage directly in the documentation, which IDEs display on hover. This is especially valuable for functions with complex parameter shapes, async flows, or non-obvious side effects.

## Bad

```js
/**
 * Applies transformations to the dataset.
 *
 * @param {Dataset} data - The dataset to transform
 * @param {Object} config - Transformation config
 * @returns {Dataset} Transformed data
 */
export function transformData(data, config) {
  // 100 lines of complex transformation logic
  // User must read all of it to understand valid config values
}
```

## Good

```js
/**
 * Applies transformations to the dataset.
 *
 * @param {Dataset} data - The dataset to transform
 * @param {Object} config - Transformation config
 * @param {'sum'|'average'|'count'} config.aggregation - Aggregation method
 * @param {string[]} [config.groupBy] - Columns to group by
 * @param {string[]} [config.sortBy] - Columns to sort by
 * @returns {Dataset} Transformed data
 *
 * @example
 * ```js
 * // Group sales by region and calculate totals
 * const result = transformData(salesData, {
 *   aggregation: 'sum',
 *   groupBy: ['region'],
 *   sortBy: ['total'],
 * });
 * // Returns: [
 * //   { region: 'West', total: 5000 },
 * //   { region: 'East', total: 3200 },
 * // ]
 * ```
 *
 * @example
 * ```js
 * // Count orders by status
 * const result = transformData(orders, {
 *   aggregation: 'count',
 *   groupBy: ['status'],
 * });
 * ```
 */
export function transformData(data, config) {
  // Implementation
}
```

## When Exceptions Apply

Simple functions (single parameter, obvious return type) don't need examples. Use `@example` when the function has non-trivial setup, multiple configuration options, or produces output whose shape isn't obvious.

## See Also

- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
- [doc-param-return](./doc-param-return.md) - Parameter documentation
