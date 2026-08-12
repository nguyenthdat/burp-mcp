# test-node-test-runner

> Use Node.js built-in test runner (`node --test`) for unit tests

## Why It Matters

Node.js 18+ includes a built-in test runner with `describe`/`it` syntax, assertion library, mocking, code coverage, and TAP output. It requires zero dependencies, eliminates version conflicts with third-party runners, and is maintained by the Node.js core team. For most projects, it replaces Jest, Mocha, and Ava.

## Bad

```js
// External dependencies — adds maintenance burden
import { describe, it } from 'mocha';
import { expect } from 'chai';

// Jest requires configuration, transforms, and a heavy dependency
// jest.config.js
// babel.config.js
// etc.
```

## Good

```js
// Zero-dependency — just run: node --test
import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

describe('UserService', () => {
  it('should create a user', () => {
    const user = createUser({ name: 'Alice' });
    assert.strictEqual(user.name, 'Alice');
  });

  it('should throw on invalid input', () => {
    assert.throws(() => createUser(null), TypeError);
  });
});
```

## Async Tests

```js
import { describe, it, before, after } from 'node:test';

describe('Database', () => {
  before(async () => {
    await connectToDatabase();
  });

  it('should fetch user', async () => {
    const user = await findUser(1);
    assert.ok(user);
  });

  after(async () => {
    await disconnect();
  });
});
```

## Running Tests

```bash
# Run all tests
node --test

# Run specific file
node --test tests/user-service.test.js

# With coverage (Node 22+)
node --test --experimental-test-coverage

# Watch mode (Node 22+)
node --test --watch
```

## When Exceptions Apply

Use Vitest/Jest when you need extensive mocking (jest.mock), snapshot testing, or React/Vue component testing. Use Playwright/Cypress for end-to-end browser tests.

## See Also

- [test-arrange-act-assert](./test-arrange-act-assert.md) - Test structure
- [test-describe-it](./test-describe-it.md) - Group tests
- [test-coverage-threshold](./test-coverage-threshold.md) - Coverage in CI
