# test-integration-vs-unit

> Balance the test pyramid between unit and integration tests

## Why It Matters

A suite made entirely of unit tests with every dependency mocked can pass while the system is broken end-to-end, because the mocks encode assumptions about collaborators that have quietly drifted out of sync. A suite made entirely of integration tests is slow, flaky, and makes it hard to pinpoint which component regressed. The test pyramid — many fast unit tests, fewer integration tests, a handful of end-to-end tests — balances speed and confidence: unit tests catch logic errors immediately, integration tests catch wiring and contract errors that mocks hide.

## Bad

```typescript
// Everything is a "unit" test with every collaborator mocked —
// nothing ever verifies these pieces actually work together.
it("should process an order", async () => {
  const repo = { save: vi.fn().mockResolvedValue(undefined) };
  const payments = { charge: vi.fn().mockResolvedValue({ ok: true }) };
  const emailer = { send: vi.fn().mockResolvedValue(undefined) };

  await processOrder({ repo, payments, emailer }, order);

  expect(repo.save).toHaveBeenCalled();
  expect(payments.charge).toHaveBeenCalled();
  expect(emailer.send).toHaveBeenCalled();
  // Never verifies the real payments client accepts this order shape,
  // or that repo.save actually persists retrievable data.
});
```

## Good

```typescript
// Unit test: pure business logic, fully isolated, runs in milliseconds
it("should reject an order with a negative quantity", () => {
  expect(() => validateOrder({ quantity: -1 })).toThrow("InvalidQuantity");
});

// Integration test: real repository against a test database,
// verifies the contract actually holds
it("should persist and retrieve an order with its line items", async () => {
  const repo = new PostgresOrderRepository(testDb);
  const saved = await repo.save(buildOrder());

  const found = await repo.findById(saved.id);
  expect(found).toEqual(saved);
});
```

## The Pyramid

| Layer | Speed | Scope | Typical count |
|---|---|---|---|
| Unit | Milliseconds | Pure functions, single class, all collaborators faked | Hundreds–thousands |
| Integration | Seconds | Real DB/queue/HTTP client against a test instance, module boundaries | Dozens–hundreds |
| End-to-end | Seconds–minutes | Full stack through the real API/UI | A handful of critical paths |

Use unit tests for branching logic and edge cases; reserve integration tests for verifying that your code and a real dependency (database driver, HTTP client, message queue) agree on the contract. If a bug keeps reaching production despite green unit tests, that's a signal the pyramid is too heavy on mocks and needs more integration coverage at the boundary that failed.

## See Also

- [test-mock-boundaries](test-mock-boundaries.md) - Mock external boundaries (network, filesystem, clock), not internal implementation details
- [test-coverage-meaningful](test-coverage-meaningful.md) - Target meaningful coverage of behavior, not a 100% coverage vanity metric
- [test-test-doubles](test-test-doubles.md) - Choose the right test double: stub, spy, mock, or fake
