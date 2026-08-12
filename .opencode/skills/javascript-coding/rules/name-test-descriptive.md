# name-test-descriptive

> Use `should`/`when`/`given` format for test names that describe behavior, not implementation

## Why It Matters

Test names are documentation. When a test fails 6 months later, the name should tell you what behavior broke without reading the test body. Names like `test1` or `works correctly` are useless. Descriptions like `should return 404 when user not found` immediately convey the contract being tested.

## Bad

```js
// Vague — doesn't describe behavior
test('test1');
test('user test');
test('works');
it('should handle it');

// Implementation-focused — fragile
test('calls findById with correct ID');
test('sets res.status to 200');
```

## Good

```js
// Behavior-focused — describes what the system does
test('should return user by ID');
test('should return 404 when user is not found');
test('should reject invalid email addresses');
test('should create a session token on successful login');

// Using describe blocks for grouping
describe('UserService', () => {
  describe('findById', () => {
    it('should return the user when found');
    it('should return null when user does not exist');
    it('should throw when ID is not a string');
  });

  describe('createUser', () => {
    it('should create a new user with valid data');
    it('should reject duplicate email addresses');
    it('should hash the password before storing');
  });
});
```

## Test Name Patterns

```js
// Pattern: "should [behavior] when [condition]"
test('should apply discount when cart total exceeds $100');
test('should return empty array when no results match');

// Pattern: "should [behavior] given [input]"
test('should parse valid ISO date string');
test('should throw TypeError given null input');

// Pattern: "should [behavior]"
test('should send confirmation email after order is placed');
test('should expire tokens after 24 hours');
```

## When Exceptions Apply

In property-based tests or generated tests, the test name may be generated automatically. For snapshot tests, the framework typically generates the name.

## See Also

- [test-describe-it](./test-describe-it.md) - Group tests with describe/it
- [test-arrange-act-assert](./test-arrange-act-assert.md) - Test structure
