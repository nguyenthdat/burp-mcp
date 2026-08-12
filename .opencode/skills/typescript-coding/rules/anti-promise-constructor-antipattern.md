# anti-promise-constructor-antipattern

> Don't wrap an already-promise-returning call in `new Promise`

## Why It Matters

`new Promise((resolve, reject) => { ... })` exists specifically to *convert* a callback-based API into a Promise; wrapping a function that already returns a Promise in another `new Promise` is redundant and dangerous, because it's easy to forget to attach a `.catch`/rejection path inside the executor, silently swallowing errors that the inner Promise would have propagated correctly on its own. It also creates an extra microtask tick with no benefit, and it means two independent rejection paths exist (the inner Promise's and the outer wrapper's) that must be kept in sync manually instead of just returning the inner Promise directly.

## Bad

```typescript
function fetchUser(id: string): Promise<User> {
  return new Promise((resolve, reject) => {
    apiClient.getUser(id) // apiClient.getUser already returns a Promise
      .then((user) => resolve(user))
      .catch((err) => reject(err)); // easy to typo, forget, or mis-wire this
  });
}

function delayedFetch(id: string) {
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      fetchUser(id).then(resolve); // forgot to forward rejections here — silent hang on error
    }, 100);
  });
}
```

## Good

```typescript
function fetchUser(id: string): Promise<User> {
  return apiClient.getUser(id); // already a Promise — just return it
}

function delayedFetch(id: string): Promise<User> {
  return new Promise((resolve, reject) => {
    // new Promise() is justified here: setTimeout is callback-based, not Promise-based
    setTimeout(() => {
      fetchUser(id).then(resolve, reject); // forward both success and failure
    }, 100);
  });
}

// Even better: use a Promise-based delay helper and async/await throughout
import { setTimeout as delay } from 'node:timers/promises';

async function delayedFetch(id: string): Promise<User> {
  await delay(100);
  return fetchUser(id); // no manual executor needed at all
}
```

## When `new Promise` Is Legitimate

Reach for `new Promise` only to adapt a genuinely callback-based or event-based API (an EventEmitter, a legacy Node.js `(err, result)` callback, `setTimeout`) into a Promise — never to re-wrap something that already returns one.

## See Also

- [async-await-over-then](async-await-over-then.md) - Prefer `async`/`await` over chained `.then()` calls
- [async-immediately-invoked](async-immediately-invoked.md) - Use IIFEs correctly for scoping in async contexts
- [err-async-propagation](err-async-propagation.md) - Let async errors propagate through rejection, not callbacks
