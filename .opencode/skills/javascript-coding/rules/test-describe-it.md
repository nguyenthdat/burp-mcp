# test-describe-it

> Group related tests with `describe` blocks and use `it`/`test` for individual cases

## Why It Matters

`describe` blocks create a hierarchy that mirrors the module structure, making test output navigable. `it` blocks (or `test`) define individual test cases. Together they produce readable output that acts as documentation. Nesting describes allows shared setup via `beforeEach` without polluting other test groups.

## Bad

```js
// No grouping — flat list of tests, hard to understand scope
test('createUser with valid data returns user');
test('createUser with duplicate email throws');
test('createUser with missing name throws');
test('findById returns user when found');
test('findById returns null when not found');
test('deleteUser removes user');
test('deleteUser throws when not found');
```

## Good

```js
// Nested describes — structure mirrors the module
describe('UserService', () => {
  describe('createUser', () => {
    it('should return the created user with valid data');
    it('should throw ValidationError when email is duplicate');
    it('should throw ValidationError when name is missing');
    it('should hash the password');
  });

  describe('findById', () => {
    it('should return the user when found');
    it('should return null when user does not exist');
    it('should throw TypeError when id is not a string');
  });

  describe('deleteUser', () => {
    it('should remove the user');
    it('should throw NotFoundError when user does not exist');
  });
});
```

## Shared Setup with beforeEach

```js
describe('OrderService', () => {
  let user;
  let product;

  beforeEach(async () => {
    user = await createTestUser();
    product = await createTestProduct({ price: 50 });
  });

  describe('createOrder', () => {
    it('should create an order for valid items', async () => {
      const order = await createOrder(user.id, [{ productId: product.id, quantity: 2 }]);
      assert.strictEqual(order.total, 100);
    });
  });
});
```

## When Exceptions Apply

For very small modules with 1-2 tests, flat `test()` calls are fine. Use `describe` when there are 3+ related tests or shared setup.

## See Also

- [test-arrange-act-assert](./test-arrange-act-assert.md) - Test structure
- [test-setup-teardown](./test-setup-teardown.md) - beforeEach/afterEach
