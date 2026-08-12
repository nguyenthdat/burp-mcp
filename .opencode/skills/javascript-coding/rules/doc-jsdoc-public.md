# doc-jsdoc-public

> Use JSDoc comments for all public API functions

## Why It Matters

JSDoc provides structured documentation that IDEs display on hover, generates API docs automatically, and enables TypeScript-style type checking in plain JavaScript. Public functions without documentation become black boxes that teammates must read the source to understand. A `@description`, `@param`, and `@returns` is the minimum.

## Bad

```js
// No documentation — consumers must read the source
export function processOrder(order, options) {
  // 50 lines of logic
}

export class UserService {
  async find(query) { /* ... */ }
}
```

## Good

```js
/**
 * Processes an order through the fulfillment pipeline.
 *
 * Validates the order, reserves inventory, charges payment,
 * and triggers shipping. Returns the updated order with
 * a tracking number on success.
 *
 * @param {Order} order - The order to process
 * @param {ProcessOptions} [options] - Processing configuration
 * @param {boolean} [options.sendEmail=true] - Send confirmation email
 * @returns {Promise<Order>} The processed order with tracking
 * @throws {ValidationError} If the order fails validation
 * @throws {PaymentError} If the payment fails
 */
export async function processOrder(order, options = {}) {
  // Implementation
}
```

## Class Documentation

```js
/**
 * Manages user accounts including creation, authentication,
 * and profile management.
 *
 * @example
 * ```js
 * const service = new UserService(db);
 * const user = await service.create({ name: 'Alice' });
 * ```
 */
export class UserService {
  /**
   * @param {Database} db - Database connection
   */
  constructor(db) {
    this.db = db;
  }

  /**
   * Finds a user by search criteria.
   *
   * @param {UserQuery} query - Search criteria
   * @returns {Promise<User|null>} The matching user or null
   */
  async find(query) {
    return this.db.users.findOne(query);
  }
}
```

## When Exceptions Apply

Private/internal functions (not exported) don't need JSDoc but benefit from inline comments for complex logic. Test files don't need JSDoc on test functions.

## See Also

- [doc-param-return](./doc-param-return.md) - Document parameters and returns
- [doc-type-annotations](./doc-type-annotations.md) - JSDoc type annotations
