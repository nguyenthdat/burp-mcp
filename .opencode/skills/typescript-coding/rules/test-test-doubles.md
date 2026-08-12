# test-test-doubles

> Choose the right test double: stub, spy, mock, or fake

## Why It Matters

"Just mock it" is not a strategy — using a strict mock where a simple stub would do adds unnecessary assertions on call order and arguments that make tests brittle against harmless refactors, while using a stub where a spy is needed hides the fact that a side effect never happened. Picking the right kind of test double keeps each test's intent clear: is it checking a value came back correctly, that something was called, or that a whole dependency's behavior was faithfully approximated?

## Bad

```typescript
import { vi } from "vitest";

// A heavyweight mock with strict call-order expectations,
// used just to return a canned value — overkill and brittle.
const emailer = {
  send: vi.fn(),
};
emailer.send.mockImplementation(() => Promise.resolve({ id: "1" }));

await registerUser(emailer, { email: "a@b.com" });

expect(emailer.send).toHaveBeenCalledTimes(1);
expect(emailer.send).toHaveBeenNthCalledWith(1, expect.anything());
expect(emailer.send).toHaveBeenCalledBefore(someOtherMock); // asserts irrelevant ordering
```

## Good

```typescript
import { vi } from "vitest";

// A stub: just returns a canned value, no behavioral assertions
const emailer = { send: vi.fn().mockResolvedValue({ id: "1" }) };
const user = await registerUser(emailer, { email: "a@b.com" });
expect(user.email).toBe("a@b.com");

// A spy: verifies the side effect happened, without over-specifying detail
expect(emailer.send).toHaveBeenCalledWith(expect.objectContaining({ to: "a@b.com" }));
```

## Test Double Cheat Sheet

| Double | Purpose | Example use |
|---|---|---|
| **Stub** | Returns canned data; no assertions on how it's called | `repo.findById = vi.fn().mockResolvedValue(user)` |
| **Spy** | Wraps a real or fake implementation and records calls for later assertion | `vi.spyOn(logger, "warn")` |
| **Mock** | A strict double with pre-set expectations on calls/arguments/order | Reserve for verifying a required side effect (e.g. "payment was charged exactly once") |
| **Fake** | A lightweight working implementation (in-memory DB, in-memory queue) | `new InMemoryUserRepository()` for integration-style tests |

Prefer fakes for anything with meaningful internal state (repositories, queues) — they let tests exercise real logic (uniqueness checks, ordering) without a mock's brittle call-order assertions.

## See Also

- [test-mock-boundaries](test-mock-boundaries.md) - Mock external boundaries (network, filesystem, clock), not internal implementation details
- [test-fixture-factories](test-fixture-factories.md) - Use factory functions to build test fixtures instead of duplicating literals
- [test-integration-vs-unit](test-integration-vs-unit.md) - Balance the test pyramid between unit and integration tests
