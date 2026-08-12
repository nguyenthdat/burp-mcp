# test-setup-teardown

> Use `before`/`beforeEach` and `after`/`afterEach` hooks instead of manual setup in each test

## Why It Matters

Repeating setup logic in every test bloats the test file and hides the unique aspects of each test case. Setup/teardown hooks centralize common preparation and cleanup, making each test focused on the behavior being verified. Teardown hooks guarantee cleanup even when tests fail, preventing cascading failures.

## Bad

```js
// Repeated setup and cleanup in every test
test('should create user', async () => {
  const db = await connectToTestDB();
  await db.clear();
  const service = new UserService(db);

  const user = await service.create({ name: 'Alice' });

  assert.strictEqual(user.name, 'Alice');
  await db.disconnect();
});

test('should find user by ID', async () => {
  const db = await connectToTestDB();
  await db.clear();
  const service = new UserService(db);
  await service.create({ name: 'Bob' });

  const user = await service.findById(1);

  assert.strictEqual(user.name, 'Bob');
  await db.disconnect();
});
```

## Good

```js
// Centralized setup and teardown
describe('UserService', () => {
  let db;
  let service;

  before(async () => {
    db = await connectToTestDB();
  });

  beforeEach(async () => {
    await db.clear();
    service = new UserService(db);
  });

  after(async () => {
    await db.disconnect();
  });

  test('should create user', async () => {
    const user = await service.create({ name: 'Alice' });
    assert.strictEqual(user.name, 'Alice');
  });

  test('should find user by ID', async () => {
    await service.create({ name: 'Bob' });
    const user = await service.findById(1);
    assert.strictEqual(user.name, 'Bob');
  });
});
```

## Hook Scope

```js
// before/after: runs once per describe block
before(() => { /* Connect to DB — once */ });

// beforeEach/afterEach: runs before/after EACH test
beforeEach(() => { /* Clear data — each test */ });

// Nested describes inherit parent hooks
describe('outer', () => {
  before(() => { /* Runs before inner tests too */ });

  describe('inner', () => {
    before(() => { /* Runs before inner tests */ });
    // Both outer.before and inner.before run
  });
});
```

## When Exceptions Apply

If only one test needs specific setup, putting it in the test is cleaner than creating a nested describe with its own before. Balance DRY with readability.

## See Also

- [test-isolation](./test-isolation.md) - Tests must not share state
- [test-describe-it](./test-describe-it.md) - Group tests with describe/it
