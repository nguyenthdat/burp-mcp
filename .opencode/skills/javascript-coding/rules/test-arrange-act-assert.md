# test-arrange-act-assert

> Structure tests with three clear sections: arrange (setup), act (execute), assert (verify)

## Why It Matters

The Arrange-Act-Assert (AAA) pattern creates a consistent test structure that makes it immediately clear what's being tested, what the setup is, and what the expected outcome is. Arbitrary ordering of setup and assertions makes tests hard to read and maintain. A blank line between sections enhances scanability.

## Bad

```js
// Mixed setup, action, and assertions — hard to follow
test('create order', () => {
  assert.strictEqual(order.total, 100);
  const order = createOrder({ items: [{ price: 50 }, { price: 50 }] });
  const user = createTestUser();
  assert.strictEqual(order.status, 'pending');
  order.userId = user.id;
});
```

## Good

```js
// Clear AAA structure
test('should calculate order total from items', () => {
  // Arrange
  const user = createTestUser();
  const items = [{ price: 50 }, { price: 30 }, { price: 20 }];

  // Act
  const order = createOrder({ userId: user.id, items });

  // Assert
  assert.strictEqual(order.total, 100);
  assert.strictEqual(order.status, 'pending');
  assert.strictEqual(order.userId, user.id);
});
```

## Multiple Related Assertions

```js
test('should create user with hashed password', () => {
  // Arrange
  const input = { email: 'alice@example.com', password: 'secret123' };

  // Act
  const user = await createUser(input);

  // Assert
  assert.strictEqual(user.email, input.email);
  assert.notStrictEqual(user.password, input.password);  // Hashed
  assert.ok(user.password.startsWith('$2b$'));           // bcrypt format
  assert.ok(user.id);
  assert.ok(user.createdAt instanceof Date);
});
```

## When Exceptions Apply

In simple one-liner tests, the sections merge naturally. The pattern is most valuable for tests with non-trivial setup or multiple assertions.

```js
test('should return 0 for empty array', () => {
  assert.strictEqual(sum([]), 0);  // Arrange + Act + Assert in one line
});
```

## See Also

- [test-describe-it](./test-describe-it.md) - Group tests with describe/it
- [test-isolation](./test-isolation.md) - Tests must not share state
