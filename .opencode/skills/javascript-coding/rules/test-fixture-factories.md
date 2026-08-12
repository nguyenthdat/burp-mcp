# test-fixture-factories

> Use factory functions for test data instead of hardcoded fixtures or inline objects

## Why It Matters

Hardcoded test data scattered across tests creates maintenance burden: changing a model requires updating dozens of tests. Factory functions centralize test data creation, provide sensible defaults, allow per-test overrides, and make tests more readable by hiding irrelevant data behind descriptive function names.

## Bad

```js
// Hardcoded data duplicated across tests
test('should create user', () => {
  const user = await createUser({
    name: 'Alice',
    email: 'alice@example.com',
    role: 'user',
    createdAt: new Date(),
    updatedAt: new Date(),
    settings: { theme: 'light', notifications: true },
    addresses: [],
  });
});

test('should update user', () => {
  const user = await createUser({
    name: 'Bob',
    email: 'bob@example.com',
    role: 'admin',
    createdAt: new Date(),
    updatedAt: new Date(),
    settings: { theme: 'dark', notifications: false },
    addresses: [],
  });
});

// Changing the User model breaks every test
```

## Good

```js
// Factory function — centralized, overridable
function createTestUser(overrides = {}) {
  return {
    name: 'Test User',
    email: `test-${crypto.randomUUID()}@example.com`,
    role: 'user',
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01'),
    settings: { theme: 'light', notifications: true },
    addresses: [],
    ...overrides,
  };
}

test('should create user with valid data', async () => {
  const user = await createUser(createTestUser());
  assert.strictEqual(user.role, 'user');
});

test('should flag admin users', async () => {
  const user = await createUser(createTestUser({ role: 'admin' }));
  assert.strictEqual(user.role, 'admin');
});
```

## Factory Composition

```js
function createTestOrder(overrides = {}) {
  const user = createTestUser();
  return {
    userId: user.id,
    items: [createTestItem(), createTestItem()],
    status: 'pending',
    total: 100,
    ...overrides,
  };
}

function createTestItem(overrides = {}) {
  return {
    productId: crypto.randomUUID(),
    quantity: 1,
    price: 50,
    ...overrides,
  };
}
```

## When Exceptions Apply

For tests that need very specific data (e.g., edge cases with exactly one field), inline data is clearer than a factory with many overrides. Use factories when the same base data appears in 3+ tests.

## See Also

- [test-isolation](./test-isolation.md) - Tests must not share state
- [test-arrange-act-assert](./test-arrange-act-assert.md) - Test structure
