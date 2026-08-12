# test-mock-over-stub

> Mock behaviors (interactions), not implementation details

## Why It Matters

Tests that verify internal implementation (which method was called, with what arguments) are fragile — they break on refactoring even when behavior is correct. Tests should verify outcomes (what the function returns, what side effects occur). Mock at module boundaries, not at the function-internal level.

## Bad

```js
// Testing implementation — fragile
test('should call database.insertOne with correct args', async () => {
  const insertSpy = mock.method(db, 'insertOne');

  await createUser({ name: 'Alice' });

  assert.strictEqual(insertSpy.mock.callCount(), 1);
  assert.deepStrictEqual(insertSpy.mock.calls[0].arguments[0], { name: 'Alice' });
  // Tests break if we switch to insertMany, upsert, or a different library
});
```

## Good

```js
// Testing behavior — robust
test('should return the created user', async () => {
  // Mock at the boundary
  mock.method(db, 'insertOne', () => ({ insertedId: '123' }));

  const user = await createUser({ name: 'Alice' });

  assert.strictEqual(user.id, '123');
  assert.strictEqual(user.name, 'Alice');
});

test('should throw if email already exists', async () => {
  mock.method(db, 'insertOne', () => {
    throw new Error('duplicate key');
  });

  await assert.rejects(
    () => createUser({ email: 'existing@example.com' }),
    /duplicate/,
  );
});
```

## Mock at Boundaries

```js
// Mock external dependencies, not internal functions
test('should send notification on order creation', async () => {
  // Mock the email service — it's an external boundary
  const sendMock = mock.method(emailService, 'send');

  await createOrder({ /* ... */ });

  assert.strictEqual(sendMock.mock.callCount(), 1);  // Behavior: email WAS sent
});

// Don't mock internal helper functions
// Bad: mock.method(module, '_calculateTax') — tests implementation
```

## When Exceptions Apply

Interaction testing is valuable when the contract IS the interaction (e.g., `analytics.track(event)` must be called with specific event data). For these cases, verifying the call arguments tests the correct thing.

## See Also

- [test-isolation](./test-isolation.md) - Tests must not share state
- [test-fixture-factories](./test-fixture-factories.md) - Factory functions for test data
