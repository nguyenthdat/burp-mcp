# test-isolation

> Tests must not depend on each other or on shared mutable state

## Why It Matters

Test interdependency creates flaky tests — tests that pass or fail depending on execution order. This erodes trust in the test suite. Each test should set up its own state and clean up after itself. When tests share state, a failure in one test cascades to others, making debugging exponentially harder.

## Bad

```js
// Tests share mutable state — order-dependent
let users = [];

test('adds a user', () => {
  users.push({ id: 1, name: 'Alice' });
  assert.strictEqual(users.length, 1);
});

test('removes a user', () => {
  // Relies on previous test adding a user — fragile!
  users = users.filter(u => u.id !== 1);
  assert.strictEqual(users.length, 0);
});
```

## Good

```js
// Each test creates its own state
test('adds a user', () => {
  const users = [];
  users.push({ id: 1, name: 'Alice' });
  assert.strictEqual(users.length, 1);
});

test('removes a user', () => {
  const users = [{ id: 1, name: 'Alice' }];  // Independent setup
  const filtered = users.filter(u => u.id !== 1);
  assert.strictEqual(filtered.length, 0);
});

// Use beforeEach for shared setup, not shared state between tests
describe('UserStore', () => {
  let store;

  beforeEach(() => {
    store = new UserStore();  // Fresh instance for each test
  });

  test('adds a user', () => {
    store.add({ id: 1 });
    assert.strictEqual(store.count(), 1);
  });

  test('removes a user', () => {
    store.add({ id: 1 });
    store.remove(1);
    assert.strictEqual(store.count(), 0);
  });
});
```

## When Exceptions Apply

Read-only shared state (constants, configuration) is fine to share between tests. Mutable databases should use transactions or be reset between tests.

## See Also

- [test-setup-teardown](./test-setup-teardown.md) - beforeEach/afterEach
- [test-fast-parallel](./test-fast-parallel.md) - Run tests in parallel
