# test-isolate-tests

> Keep tests isolated and order-independent, with no shared mutable state

## Why It Matters

A test suite that passes only when run in a specific order is really one giant test wearing many names — it's fragile under parallelization, breaks when someone adds a new test in the "wrong" place, and makes failures nearly impossible to reproduce in isolation (`vitest run my.test.ts` behaves differently from the full suite). Shared mutable state, whether a module-level array, a singleton database connection with leftover rows, or a `beforeAll` that seeds data for the whole file, is the most common cause. Each test should be runnable alone, in any order, and repeatedly, with the same result.

## Bad

```typescript
import { describe, expect, it } from "vitest";
import { UserStore } from "./user-store";

// Module-level state shared across every test in the file
const store = new UserStore();

describe("UserStore", () => {
  it("should add a user", () => {
    store.add({ id: "1", name: "Ana" });
    expect(store.all()).toHaveLength(1);
  });

  it("should list users", () => {
    // Passes only because the previous test happened to run first
    expect(store.all()).toHaveLength(1);
  });
});
```

## Good

```typescript
import { beforeEach, describe, expect, it } from "vitest";
import { UserStore } from "./user-store";

describe("UserStore", () => {
  let store: UserStore;

  beforeEach(() => {
    store = new UserStore(); // fresh state for every test
  });

  it("should add a user", () => {
    store.add({ id: "1", name: "Ana" });
    expect(store.all()).toHaveLength(1);
  });

  it("should start empty", () => {
    expect(store.all()).toHaveLength(0);
  });
});
```

## Common Patterns

- Recreate stateful dependencies in `beforeEach`, not `beforeAll` — `beforeAll` setup is shared and therefore mutable across tests in the block.
- For integration tests against a real database, wrap each test in a transaction that rolls back afterward, or truncate tables in `afterEach`.
- Run the suite with `--sequence.shuffle` (Vitest) or `--randomize` occasionally in CI to catch hidden order dependencies before they surface in production regressions.
- Avoid module-level `let` variables mutated by tests; prefer returning fresh instances from a factory function per test.

## See Also

- [test-fixture-factories](test-fixture-factories.md) - Use factory functions to build test fixtures instead of duplicating literals
- [anti-global-mutable-state](anti-global-mutable-state.md) - avoid shared mutable state in production code for the same reasons
- [test-arrange-act-assert](test-arrange-act-assert.md) - Structure tests as arrange/act/assert
