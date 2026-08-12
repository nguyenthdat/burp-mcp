# mod-no-barrel-re-export-star

> Avoid `export * from` in library code — it makes the API surface ambiguous

## Why It Matters

`export * from './module.js'` re-exports everything from another module, including unintended symbols. This leaks internal implementation details, makes it hard to trace where a symbol originates, and can cause name collisions. Explicit re-exports make the public API intentional and auditable.

## Bad

```js
// Re-exports everything — leaks internal symbols
export * from './user-service.js';
export * from './order-service.js';
export * from './internal-helpers.js';  // Internal helpers now part of public API

// Consumer sees everything mixed together
import { createUser, cancelOrder, fetchFromCache, retryWithBackoff } from './index.js';
// Where did retryWithBackoff come from? Hard to trace.
```

## Good

```js
// Explicit re-exports — intentional API surface
export { createUser, updateUser, deleteUser } from './user-service.js';
export { createOrder, cancelOrder, getOrder } from './order-service.js';
// Internal helpers are NOT re-exported

// Consumer sees a clean, curated API
import { createUser, cancelOrder } from './index.js';
```

## When `export *` Is Acceptable

```js
// 1. Re-exporting ALL of a well-defined namespace
export * as UserService from './user-service.js';
// Consumer: UserService.createUser()

// 2. Type-only re-exports in TypeScript (not applicable in JS)

// 3. Re-exporting from a prelude/utilities module where all exports are public
export * from './constants.js';  // All constants are intentionally public
```

## When Exceptions Apply

Use `export *` for namespace re-exports (`export * as Foo`) where all symbols are intentionally public. Avoid for flat re-exports.

## See Also

- [mod-barrel-files](./mod-barrel-files.md) - Barrel file patterns
- [mod-named-over-default](./mod-named-over-default.md) - Named vs default exports
