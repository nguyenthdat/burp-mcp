# test-fast-parallel

> Keep unit tests fast (< 100ms each) and run them in parallel

## Why It Matters

A slow test suite kills developer productivity. When tests take minutes to run, developers run them less often, leading to larger batches of changes and harder debugging. Fast tests encourage frequent runs. Parallel execution leverages multi-core machines. The Node.js test runner runs tests concurrently by default.

## Bad

```js
// Slow test — 2 second timeout
test('should process data', async () => {
  await new Promise(resolve => setTimeout(resolve, 2000));
  const result = process(data);
  assert.ok(result);
});

// Sequential execution
test('test 1', async () => {
  await heavyOperation();  // 5 seconds
});

test('test 2', async () => {
  await heavyOperation();  // 5 seconds
});
// Total: 10 seconds
```

## Good

```js
// Fast, focused test
test('should process data', () => {
  const result = process(data);
  assert.ok(result);
});

// Node.js test runner runs files in parallel
// Each file is its own worker thread

// Mock slow external dependencies
test('should save user', async () => {
  mock.method(db, 'insert', () => Promise.resolve({ id: '123' }));

  const user = await createUser({ name: 'Alice' });

  assert.strictEqual(user.id, '123');
});

// Use fake timers for time-dependent tests
import { mock } from 'node:test';

test('should expire after timeout', async () => {
  mock.timers.enable();

  const promise = withTimeout(slowOperation(), 5000);

  mock.timers.tick(5000);
  await assert.rejects(promise, /timeout/);

  mock.timers.reset();
});
```

## Performance Targets

```js
// Unit tests: < 100ms per test
// Integration tests: < 500ms per test
// E2E tests: < 5s per test

// Run subsets during development
// $ node --test --test-name-pattern="UserService"
// $ node --test tests/unit/
```

## When Exceptions Apply

Integration tests that need real databases, file I/O, or network calls are inherently slower. Keep them in a separate directory and run them less frequently (e.g., on commit, not on every save).

## See Also

- [test-node-test-runner](./test-node-test-runner.md) - Node.js built-in test runner
- [test-integration-separate](./test-integration-separate.md) - Separate test types
