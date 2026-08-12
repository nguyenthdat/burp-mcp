# fn-avoid-delete-mutation

> Avoid `delete` on objects — set properties to `undefined` or create new objects without the property

## Why It Matters

The `delete` operator removes a property from an object, which deoptimizes the object in V8 (it changes the object's "hidden class"), making property access slower. It also mutates the original object, which violates immutability. For hot paths, `delete` can degrade performance significantly.

## Bad

```js
// delete mutates and deoptimizes
function sanitize(user) {
  delete user.password;       // Mutates original + deoptimizes
  delete user.creditCard;     // Mutates original + deoptimizes
  return user;
}

// delete on arrays — leaves holes
const items = [1, 2, 3, 4];
delete items[1];  // [1, empty, 3, 4] — length unchanged
items.length;      // 4
```

## Good

```js
// Create a new object without the properties
function sanitize({ password, creditCard, ...safe }) {
  return safe;  // Returns new object without password and creditCard
}

// If you must modify in-place (avoid if possible)
function sanitize(user) {
  user.password = undefined;
  user.creditCard = undefined;
  return user;
}

// Use Map for dynamic key removal
const data = new Map(Object.entries(obj));
data.delete('password');  // Maps are optimized for deletion
```

## Delete on Arrays

```js
// Bad — delete leaves holes
delete arr[index];

// Good — use splice or filter
arr.splice(index, 1);                    // Remove by index
const filtered = arr.filter((_, i) => i !== index);  // Immutable
```

## When Exceptions Apply

`delete` is acceptable when removing keys from a `Map`, or when removing a property from an object that won't be accessed frequently afterward (e.g., cleanup after processing). Avoid in hot paths.

## See Also

- [fn-immutability](./fn-immutability.md) - Immutable patterns
- [fn-rest-spread](./fn-rest-spread.md) - Rest/spread for object manipulation
