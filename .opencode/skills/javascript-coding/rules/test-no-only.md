# test-no-only

> Never commit `.only()` or `.skip()` to the main branch

## Why It Matters

`.only()` limits test execution to a single test/describe block, silently skipping all other tests. If committed, CI passes because only one test runs — masking failures in all other tests. `.skip()` permanently disables a test, usually as a temporary measure that's forgotten. Both should be caught by pre-commit hooks.

## Bad

```js
// .only() committed — all other tests are skipped silently
describe.only('UserService', () => {
  test('creates user', () => { /* ... */ });
});

test.only('this one test', () => { /* ... */ });

// .skip() committed — test is permanently disabled
test.skip('should handle edge case', () => { /* ... */ });
describe.skip('PaymentService', () => { /* ... */ });
```

## Good

```js
// No .only or .skip — all tests run
describe('UserService', () => {
  test('creates user', () => { /* ... */ });
  test('deletes user', () => { /* ... */ });
});

// For temporarily skipping, use a TODO comment and track in an issue
// TODO(#1234): Re-enable when database migration is complete
test.skip('should handle edge case', () => { /* ... */ });

// Or use a conditional skip with a flag
test('experimental feature', { skip: !isFeatureEnabled('new-auth') }, () => {
  /* ... */
});
```

## ESLint Detection

```jsonc
// eslint.config.mjs
{
  "rules": {
    "no-only-tests/no-only-tests": "error"  // eslint-plugin-no-only-tests
  }
}
```

## Pre-commit Hook

```jsonc
// package.json
{
  "scripts": {
    "lint-staged": "lint-staged"
  },
  "lint-staged": {
    "*.test.{js,mjs}": ["eslint --no-ignore --rule 'no-only-tests/no-only-tests: error'"]
  }
}
```

## When Exceptions Apply

`.only` is fine during local development. The rule is against committing it. Use eslint-plugin-no-only-tests to catch it in CI.

## See Also

- [lint-husky-lint-staged](./lint-husky-lint-staged.md) - Pre-commit hooks
- [test-node-test-runner](./test-node-test-runner.md) - Node.js test runner
