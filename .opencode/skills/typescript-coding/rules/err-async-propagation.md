# err-async-propagation

> Let async/await propagate rejections naturally instead of mixing `.then`/`.catch`

## Why It Matters

Mixing `await` with chained `.then()`/`.catch()` in the same function makes error flow hard to reason about — some rejections are caught by the `.catch`, others bypass it and need a surrounding `try/catch`, and it's easy to lose track of which is which. Using `async/await` consistently lets a normal `try/catch` capture every rejection in the function, including ones from `await`ed calls nested inside loops or conditionals, with the same control flow you'd use for synchronous exceptions.

## Bad

```typescript
async function processOrder(orderId: string) {
  const order = await fetchOrder(orderId);

  return chargeCard(order.paymentMethod, order.total)
    .then((receipt) => {
      return sendConfirmation(order.customerEmail, receipt);
    })
    .catch((err) => {
      // Only catches chargeCard/sendConfirmation failures, NOT fetchOrder failures above
      logger.error("payment flow failed", err);
      throw err;
    });
}
```

## Good

```typescript
async function processOrder(orderId: string) {
  try {
    const order = await fetchOrder(orderId);
    const receipt = await chargeCard(order.paymentMethod, order.total);
    await sendConfirmation(order.customerEmail, receipt);
    return receipt;
  } catch (err) {
    // Catches failures from every awaited call above, in one place
    logger.error("payment flow failed", { orderId, error: err });
    throw err;
  }
}
```

## Why the Mixed Style Is Error-Prone

```typescript
// Subtle bug: the try/catch does NOT catch rejections from the un-awaited promise
async function broken() {
  try {
    doSomethingAsync().then((result) => useResult(result)); // not awaited!
  } catch (err) {
    // Never reached if doSomethingAsync() or its .then() callback rejects
  }
}

// Fixed: await the whole chain, or better, remove the chain entirely
async function fixed() {
  try {
    const result = await doSomethingAsync();
    useResult(result);
  } catch (err) {
    // Reliably catches failures from both the await and useResult
  }
}
```

## When `.then`/`.catch` Still Make Sense

- Fire-and-forget background work where you deliberately don't want to block the caller (paired with `.catch` to avoid an unhandled rejection — see `err-unhandled-rejection`).
- Library code that must support environments without top-level `async` functions in scope.
- Short one-off chains outside of any `async` function context.

Even then, avoid interleaving `await` and `.then` on the *same* logical chain within one function.

## See Also

- [async-await-over-then](async-await-over-then.md) - Prefer async/await syntax over chained .then/.catch calls
- [err-unhandled-rejection](err-unhandled-rejection.md) - Register process-level handlers for unhandled promise rejections
- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise unawaited and unhandled
