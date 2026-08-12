# lint-max-params

> Limit function parameters to 3–4 maximum

## Why It Matters

Functions with many parameters are hard to call correctly (wrong argument order, forgotten arguments) and signal that the function does too much. An options object replaces positional parameters with named properties, making calls self-documenting and optional parameters explicit.

## Bad

```js
// Too many parameters — order-dependent, hard to call
function createUser(name, email, role, active, sendEmail, notifyAdmin, locale) {
  // ...
}

createUser('Alice', 'alice@example.com', 'admin', true, true, false, 'en-US');
// What do true, true, false mean? Which is which?
```

## Good

```js
// Options object — self-documenting, order-independent
function createUser({ name, email, role = 'user', active = true, sendEmail = false, notifyAdmin = false, locale = 'en' } = {}) {
  // ...
}

createUser({
  name: 'Alice',
  email: 'alice@example.com',
  role: 'admin',
  locale: 'en-US',
});
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'max-params': ['warn', 4],
  },
}
```

## When Exceptions Apply

Well-known callback signatures (e.g., Express middleware `(req, res, next)`, array methods `(item, index, array)`) are exempt because the convention is established.

## See Also

- [fn-destructure](./fn-destructure.md) - Destructuring parameters
- [fn-default-params](./fn-default-params.md) - Default parameter values
