# test-async-testing

> Always await or return promises in async tests — never let a test complete before its assertions

## Why It Matters

A test that creates a promise without awaiting it will appear to pass even if the promise rejects, because the test runner considers the test complete when the test function returns (not when all promises resolve). This creates false positives — tests that pass but don't actually verify anything. Always await async operations in tests.

## Bad

```js
// Test completes before the assertion — always passes
test('fetches user', () => {
  fetchUser(1).then(user => {
    assert.strictEqual(user.name, 'Alice');  // Never runs if fetchUser fails
  });
  // Test ends here — promise is unhandled
});

// Missing await — test passes incorrectly
test('saves record', () => {
  saveRecord({ id: 1 });  // Fire and forget
  assert.ok(true);  // Always passes
});
```

## Good

```js
// Await the async operation
test('fetches user', async () => {
  const user = await fetchUser(1);
  assert.strictEqual(user.name, 'Alice');
});

// Return the promise directly
test('fetches user', () => {
  return fetchUser(1).then(user => {
    assert.strictEqual(user.name, 'Alice');
  });
});

// Test rejection
test('throws on invalid ID', async () => {
  await assert.rejects(
    () => fetchUser(null),
    { name: 'TypeError' },
  );
});
```

## Detecting Missing Awaits

```js
// Linting rule (typescript-eslint)
// "require-await": "error"

// Node.js test runner detects unhandled rejections
// Test will fail with "Test did not finish within timeout" or "unhandledRejection"
```

## When Exceptions Apply

Fire-and-forget operations that don't affect the test outcome (e.g., logging in test cleanup) don't need awaiting. But prefer `await` everywhere in tests for consistency.

## See Also

- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Never create unhandled promises
- [async-error-swallowed](./async-error-swallowed.md) - Handle promise rejections
