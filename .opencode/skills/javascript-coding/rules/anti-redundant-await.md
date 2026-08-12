# anti-redundant-await

> Don't `await` non-promise values unnecessarily — it adds microtask overhead

## Why It Matters

`await` on a non-thenable value wraps it in `Promise.resolve()`, adding an unnecessary microtask to the event loop. While usually harmless, in hot loops or performance-sensitive code this adds measurable latency. More importantly, it signals to readers that the value might be a promise when it isn't, creating confusion.

## Bad

```js
// Redundant await — value is never a promise
async function getName(user) {
  return await user.name;  // user.name is a string, not a promise
}

// Redundant await in return
async function getResult() {
  const data = fetchData();
  return await data;  // data is already the resolved value
}
```

## Good

```js
// No unnecessary await
async function getName(user) {
  return user.name;
}

// Return non-promise values directly
async function getResult() {
  return fetchData();
}

// await only on actual promises
async function getConfig() {
  const response = await fetch('/api/config');  // fetch returns a promise
  return response.json();  // .json() returns a promise — either await or return
}
```

## When Exceptions Apply

`return await promise` is correct inside a `try/catch` block — it ensures the rejection is caught by the catch block. ESLint's `no-return-await` rule handles this nuance.

```js
async function loadConfig() {
  try {
    return await fetchConfig();  // Correct — catch block handles rejection
  } catch (err) {
    console.error('Config load failed:', err);
    return defaultConfig;
  }
}
```

## See Also

- [async-return-promise](./async-return-promise.md) - Return promises from async functions
- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Handle promise rejection
