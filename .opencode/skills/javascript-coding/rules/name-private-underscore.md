# name-private-underscore

> Use an underscore (`_`) prefix for module-level private variables and functions

## Why It Matters

JavaScript has no true `private` for module-level declarations (only `#` for class fields). The `_` prefix is a widely understood convention that signals "this is internal — don't use it from outside this module." Linters, bundlers, and IDEs respect this convention. It's not enforced by the runtime, but it's enforced by team discipline.

## Bad

```js
// No distinction between public and internal
export function publicAPI() { /* ... */ }
export function internalHelper() { /* ... */ }  // Looks just like publicAPI

export const CONFIG_DEFAULTS = { port: 3000 };  // Should consumers use this?
export const db = await connectToDatabase();    // Exposed connection object
```

## Good

```js
// _ prefix for module-private exports (when you must export for testing)
export function publicAPI() { /* ... */ }
export function _internalHelper() { /* ... */ }  // Testing-only export

// Better: don't export private functions at all
function internalHelper() { /* ... */ }  // Not exported — truly private

export function publicAPI() {
  internalHelper();  // Uses the non-exported private function
}

// Private module state
let _dbConnection = null;
export async function getDB() {
  if (!_dbConnection) _dbConnection = await connectToDatabase();
  return _dbConnection;
}
```

## Class Private Fields (Real Privacy)

```js
class UserService {
  #cache = new Map();  // Truly private — not accessible outside the class

  async findById(id) {
    if (this.#cache.has(id)) return this.#cache.get(id);
    const user = await db.users.findOne({ id });
    this.#cache.set(id, user);
    return user;
  }
}
```

## When Exceptions Apply

When using `#` private fields in classes, the `_` prefix is unnecessary (`#` is the real privacy mechanism). In modules, prefer not exporting private functions at all rather than using the `_` prefix.

## See Also

- [mod-export-near-definition](./mod-export-near-definition.md) - Export near definitions
- [mod-side-effect-free](./mod-side-effect-free.md) - Side-effect-free modules
