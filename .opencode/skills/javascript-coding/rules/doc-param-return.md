# doc-param-return

> Always document `@param` and `@returns` in JSDoc for public functions

## Why It Matters

`@param` and `@returns` define the function's contract: what goes in, what comes out. IDEs display this inline, providing type hints and descriptions without reading the source. Missing parameter documentation is the most common complaint about undocumented code. These tags also power TypeScript's `checkJs` mode for type checking plain JavaScript.

## Bad

```js
/**
 * Calculates the shipping cost for an order.
 */
export function calculateShipping(order, destination, carrier) {
  // Which fields of `order` are used?
  // What is `carrier`? String? Object?
  // What does this return? Number? Object?
}
```

## Good

```js
/**
 * Calculates the shipping cost for an order.
 *
 * @param {Order} order - The order to calculate shipping for
 * @param {Destination} destination - Shipping destination address
 * @param {'usps'|'ups'|'fedex'} carrier - The shipping carrier
 * @returns {number} The shipping cost in cents
 */
export function calculateShipping(order, destination, carrier) {
  // Implementation
}
```

## Optional and Default Params

```js
/**
 * Creates a new user account.
 *
 * @param {Object} input - User creation input
 * @param {string} input.email - Email address
 * @param {string} input.name - Display name
 * @param {string} [input.role='user'] - User role
 * @param {boolean} [input.sendWelcome=false] - Send welcome email
 * @returns {Promise<User>} The created user
 */
export async function createUser({ email, name, role = 'user', sendWelcome = false }) {
  // Implementation
}
```

## Multiple Return Types

```js
/**
 * Finds a record by ID.
 *
 * @param {string} id - The record ID
 * @returns {Record|null} The record if found, null otherwise
 */
export function findById(id) {
  return records.get(id) ?? null;
}
```

## When Exceptions Apply

Trivial getters and setters can omit `@returns` if the return value is obvious. Functions with 1-2 simple parameters can use a brief `@param` without full descriptions.

## See Also

- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
- [doc-type-annotations](./doc-type-annotations.md) - Type annotations
