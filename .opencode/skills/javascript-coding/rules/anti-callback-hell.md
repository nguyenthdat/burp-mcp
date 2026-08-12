# anti-callback-hell

> Don't nest callbacks deeply — use async/await or Promise chains

## Why It Matters

Deeply nested callbacks create pyramids of code that are hard to read, impossible to refactor, and error-prone. Each level of nesting adds cognitive load and makes error handling inconsistent. Async/await flattens the pyramid into a linear, readable sequence.

## Bad

```js
function processOrder(orderId, callback) {
  getOrder(orderId, (err, order) => {
    if (err) return callback(err);
    validateOrder(order, (err, valid) => {
      if (err) return callback(err);
      chargePayment(valid, (err, receipt) => {
        if (err) return callback(err);
        shipOrder(receipt, (err, tracking) => {
          if (err) return callback(err);
          callback(null, tracking);
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
  const receipt = await chargePayment(valid);
  const tracking = await shipOrder(receipt);
  return tracking;
}
```

## When Exceptions Apply

Event-driven architectures (EventEmitter, streams) naturally use callbacks. The anti-pattern is specifically about deep nesting of sequential operations.

## See Also

- [async-avoid-callback-hell](./async-avoid-callback-hell.md) - Use async/await
- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Never create unhandled promises
