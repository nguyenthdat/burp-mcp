# test-async-test-patterns

> Always await async assertions; never leave a test's promise unhandled

## Why It Matters

An `async` test function that isn't awaited by the runner, or an assertion that runs inside a `.then()` that the test doesn't wait for, can report a false pass: the test function returns before the assertion ever executes, so a thrown expectation error becomes an unhandled rejection that Vitest or Jest may log but not fail the suite on. This is one of the most common ways a broken test suite goes green. Every async operation in a test — the call under test and any assertions on its result — must be awaited or returned.

## Bad

```typescript
import { expect, it } from "vitest";
import { fetchUser } from "./users";

it("should fetch a user by id", () => {
  // Missing await: the test function returns immediately,
  // the assertion runs after the test is already marked "passed".
  fetchUser("123").then((user) => {
    expect(user.id).toBe("123");
  });
});
```

## Good

```typescript
import { expect, it } from "vitest";
import { fetchUser } from "./users";

it("should fetch a user by id", async () => {
  const user = await fetchUser("123");
  expect(user.id).toBe("123");
});

it("should reject with NotFoundError for an unknown id", async () => {
  await expect(fetchUser("missing")).rejects.toThrow("NotFoundError");
});
```

## Common Patterns

- Use `await expect(promise).rejects.toThrow(...)` instead of wrapping the call in `try/catch` — it fails loudly if the promise unexpectedly resolves.
- Enable `no-floating-promises` from `typescript-eslint` in test files too; it catches unawaited `expect().resolves` chains and forgotten `await` on the call under test.
- For callback-based APIs without a promise wrapper, use the test runner's `done` callback explicitly and call it in every code path, including error branches, or wrap the callback in a `new Promise` and `await` that.
- When testing concurrent operations, use `Promise.all([...])` inside the `await` rather than firing promises without awaiting each one, so a rejection in any branch fails the test.

## See Also

- [async-no-floating-promises](async-no-floating-promises.md) - detect and prevent unhandled promises
- [test-fake-timers](test-fake-timers.md) - Use fake timers to test time-dependent code deterministically
- [err-unhandled-rejection](err-unhandled-rejection.md) - handle rejected promises instead of letting them escape
