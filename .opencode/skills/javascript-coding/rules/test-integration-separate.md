# test-integration-separate

> Keep unit tests and integration tests in separate directories with separate run commands

## Why It Matters

Unit tests should be fast and run on every save. Integration tests are slower and may require external services. Mixing them makes the fast path slow and discourages frequent test runs. Separation allows running unit tests during development and integration tests in CI, with different frequency and resource allocation.

## Bad

```js
// All tests mixed — slow tests block fast feedback
tests/
  user-service.test.js     // Unit test (fast)
  user-service-db.test.js  // Integration test (slow — needs database)
  order-service.test.js    // Unit test (fast)
  api-endpoints.test.js    // Integration test (slow — needs server)
```

## Good

```js
tests/
  unit/
    user-service.test.js
    order-service.test.js
    email-validator.test.js
  integration/
    database.test.js
    api.test.js
    redis-cache.test.js
  e2e/
    user-flow.test.js
    checkout-flow.test.js
```

```jsonc
// package.json
{
  "scripts": {
    "test": "node --test tests/unit/",
    "test:unit": "node --test tests/unit/",
    "test:integration": "node --test tests/integration/",
    "test:e2e": "node --test tests/e2e/",
    "test:all": "npm run test:unit && npm run test:integration"
  }
}
```

## CI Pipeline

```yaml
# .github/workflows/test.yml
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npm run test:unit

  integration:
    needs: unit
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npm run test:integration
```

## When Exceptions Apply

For small projects with fewer than 20 tests, separation may be unnecessary overhead. Introduce separation when the test suite diverges in speed or dependencies.

## See Also

- [test-fast-parallel](./test-fast-parallel.md) - Keep tests fast
- [test-node-test-runner](./test-node-test-runner.md) - Node.js test runner
