# async-avoid-callback-hell

> Prefer async/await over deeply nested callbacks

## Why It Matters

Deeply nested callbacks ("callback hell") make error handling brittle, stack traces unreadable, and control flow difficult to follow. Async/await flattens asynchronous code into a linear, synchronous-looking sequence that is easier to read, debug, and maintain.

## Bad

```js
function processOrder(orderId, callback) {
  getOrder(orderId, (err, order) => {
    if (err) return callback(err);
    validateOrder(order, (err, valid) => {
      if (err) return callback(err);
      processPayment(valid, (err, receipt) => {
        if (err) return callback(err);
        sendConfirmation(receipt, (err) => {
          if (err) return callback(err);
          callback(null, receipt);
        });
      });
    });
  });
}
```

## Good

```js
async function processOrder(orderId) {
  const order = await getOrder(orderId);
  const valid = await validateOrder(order);
  const receipt = await processPayment(valid);
  await sendConfirmation(receipt);
  return receipt;
}
```

## When Exceptions Apply

When working with legacy callback-based APIs, wrap them in promisified helpers:

```js
import { promisify } from 'node:util';

const readFile = promisify(fs.readFile);
const content = await readFile(path, 'utf8');
```

## See Also

- [async-return-promise](./async-return-promise.md) - Always return promises from async functions
- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Never create unhandled promises
