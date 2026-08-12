# name-callback-descriptive

> Name callbacks descriptively — not `cb`, `fn`, or anonymous

## Why It Matters

Callbacks with descriptive names appear in stack traces and debugger views, making it possible to understand the flow of asynchronous code. A callback named `processUserData` tells you what happens; `cb` tells you nothing. Named callbacks also make stack traces meaningful in production error tracking.

## Bad

```js
// Anonymous or generic callbacks — unhelpful in stack traces
fetchUsers().then(data => processData(data));
items.forEach(item => save(item));
app.get('/users', (req, res) => { /* ... */ });

function doWork(cb) {
  asyncOperation().then(cb);
}
```

## Good

```js
// Descriptive callback names
fetchUsers().then(processUserData);
items.forEach(saveItem);
app.get('/users', handleGetUsers);

function doWork(onComplete) {
  asyncOperation().then(onComplete);
}

// Named function over inline callback
async function processUserData(data) {
  const enriched = await enrichUsers(data);
  return formatResponse(enriched);
}
```

## Callback Parameter Names

```js
// Callback parameters should describe the callback's role
function readConfig(onSuccess) { /* ... */ }
function watchFile(onChange) { /* ... */ }
function onExit(cleanup) { /* ... */ }

// Event listeners
button.addEventListener('click', onSubmitClick);
emitter.on('error', handleError);

// Array methods — use descriptive names for callbacks
users.filter(isActive);
users.map(toPublicProfile);
prices.reduce(sumTotal, 0);
```

## When Exceptions Apply

In one-liner arrow callbacks (`arr.map(x => x * 2)`), naming the callback is unnecessary overkill. Name them when the callback spans multiple lines or appears in nontrivial Promise chains.

## See Also

- [name-verb-function](./name-verb-function.md) - Verb-first function names
- [name-handler-verbs](./name-handler-verbs.md) - Handler prefixes
