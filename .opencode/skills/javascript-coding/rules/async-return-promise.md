# async-return-promise

> Always return a value or await a promise in async functions

## Why It Matters

Async functions implicitly wrap their return value in a Promise. If you call an async function without awaiting or returning it, you create a fire-and-forget scenario. The caller cannot await completion or catch errors. Always return the result or explicitly await within the function.

## Bad

```js
// Fire and forget — caller can't await or catch errors
async function saveUser(user) {
  database.insert(user);  // Missing await — returns undefined immediately
}

// Caller thinks they're awaiting but operation already leaked
const result = saveUser(data);  // result is Promise<undefined>, not Promise<row>
```

## Good

```js
async function saveUser(user) {
  return await database.insert(user);  // Explicit, caller can await
}

// Or, let the implicit return handle it
async function saveUser(user) {
  return database.insert(user);  // Returns Promise<row> automatically
}

// Caller properly awaits
const row = await saveUser(data);
```

## See Also

- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Never create unhandled promises
- [async-error-swallowed](./async-error-swallowed.md) - Handle promise rejections
