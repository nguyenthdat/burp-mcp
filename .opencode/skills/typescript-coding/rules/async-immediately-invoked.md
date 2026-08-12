# async-immediately-invoked

> Use an async IIFE to run async code in non-async contexts

## Why It Matters

Sometimes you need to run `await`-based code in a place that itself cannot be `async` — a top-level script on an older target, a synchronous callback signature you don't control, or a class field initializer. An async immediately-invoked function expression (IIFE) gives you an `async` scope to await inside, without requiring the enclosing function or module to change its own signature. Without it, developers often resort to `.then()` chains or floating promises in exactly the spots where control flow matters most (startup, event handlers).

## Bad

```typescript
// Top-level script targeting a runtime without top-level await support
fetchConfig().then((config) => {
  startServer(config);
}).catch((err) => {
  console.error(err);
  process.exit(1);
});
```

## Good

```typescript
(async () => {
  try {
    const config = await fetchConfig();
    startServer(config);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
```

## Common Use Case: Synchronous Event Handler Signature

```typescript
button.addEventListener("click", () => {
  // addEventListener's callback isn't awaited by the DOM, so make the
  // body an async IIFE to get try/await ergonomics inside it.
  void (async () => {
    try {
      await submitForm();
    } catch (err) {
      showError(err);
    }
  })();
});
```

## Naming the IIFE for Stack Traces

```typescript
void (async function loadInitialData() {
  await fetchAndRenderDashboard();
})();
// Named function expressions show up in stack traces and profiler
// output as "loadInitialData" instead of an anonymous frame.
```

## See Also

- [async-void-operator](async-void-operator.md) - Use the `void` operator to mark an intentionally ignored promise
- [async-top-level-await](async-top-level-await.md) - Use top-level `await` only at module entry points
- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise floating; await, return, or explicitly void it
