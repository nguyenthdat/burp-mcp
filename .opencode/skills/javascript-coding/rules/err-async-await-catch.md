# err-async-await-catch

> Use try/catch with await, not .catch() chains in async functions

## Why It Matters

Mixing `.catch()` callbacks with `await` creates two separate error-handling paths in the same function, making control flow harder to follow. Using `try/catch` with `await` keeps error handling in a single, linear structure that reads top-to-bottom.

## Bad

```js
// Mixing patterns — unclear what happens when
async function processOrder(id) {
  const order = await fetchOrder(id).catch(err => {
    console.error('Fetch failed:', err);
    return null;
  });

  if (!order) return;

  const validated = await validateOrder(order).catch(err => {
    console.error('Validation failed:', err);
    return null;
  });

  // Error handling is split between .catch() callbacks and the main flow
}
```

## Good

```js
async function processOrder(id) {
  try {
    const order = await fetchOrder(id);
    const validated = await validateOrder(order);
    return validated;
  } catch (err) {
    if (err instanceof NotFoundError) {
      return null;
    }
    console.error('Order processing failed:', err);
    throw err;
  }
}
```

## Partial Handling with Inner try/catch

```js
async function loadDashboard(userId) {
  try {
    const profile = await fetchProfile(userId);
    let settings;

    try {
      settings = await fetchSettings(userId);
    } catch (err) {
      console.warn('Settings unavailable, using defaults');
      settings = DEFAULT_SETTINGS;
    }

    return { profile, settings };
  } catch (err) {
    console.error('Dashboard load failed:', err);
    throw err;
  }
}
```

## When Exceptions Apply

`.catch()` is appropriate at the end of a Promise chain when you're not inside an async function:

```js
function loadData(onSuccess) {
  fetch('/api/data')
    .then(res => res.json())
    .then(onSuccess)
    .catch(err => console.error('Load failed:', err));
}
```

## See Also

- [err-try-catch-narrow](./err-try-catch-narrow.md) - Keep try blocks small
- [err-finally-cleanup](./err-finally-cleanup.md) - Use finally for cleanup
