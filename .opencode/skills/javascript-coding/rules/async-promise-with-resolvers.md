# async-promise-with-resolvers

> Use Promise.withResolvers() (ES2024) instead of the Promise constructor for deferred promises

## Why It Matters

The traditional pattern of storing `resolve`/`reject` outside the Promise constructor is verbose and easy to get wrong. `Promise.withResolvers()` returns `{ promise, resolve, reject }` in a single call, eliminating the need for a wrapper and reducing boilerplate.

## Bad

```js
// Manual deferred pattern — verbose
function createDeferred() {
  let resolve, reject;
  const promise = new Promise((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const { promise, resolve, reject } = createDeferred();

// Promise constructor with external state — error-prone
let externalResolve;
const promise = new Promise((resolve) => {
  externalResolve = resolve;
});
```

## Good

```js
// ES2024 — clean and standard
const { promise, resolve, reject } = Promise.withResolvers();

// Use in event-based patterns
function createAbortableOperation(signal) {
  const { promise, resolve, reject } = Promise.withResolvers();

  signal.addEventListener('abort', () => {
    reject(new DOMException('Aborted', 'AbortError'));
  }, { once: true });

  doAsyncWork().then(resolve, reject);
  return promise;
}

// Use for request/response pairing
function createRPC() {
  const pending = new Map();

  return {
    call(method, params) {
      const { promise, resolve, reject } = Promise.withResolvers();
      const id = crypto.randomUUID();
      pending.set(id, { resolve, reject });
      send({ id, method, params });
      return promise;
    },
    handleResponse({ id, result, error }) {
      const deferred = pending.get(id);
      if (deferred) {
        pending.delete(id);
        if (error) deferred.reject(new Error(error));
        else deferred.resolve(result);
      }
    },
  };
}
```

## When Exceptions Apply

If you're targeting Node.js < 22 or browsers without ES2024 support, use the manual deferred pattern or import a polyfill.

## See Also

- [async-abort-control](./async-abort-control.md) - Cancellable operations with AbortController
- [async-timeout-pattern](./async-timeout-pattern.md) - Timeout wrappers
