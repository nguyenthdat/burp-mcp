# test-mock-boundaries

> Mock external boundaries (network, filesystem, clock), not internal implementation details

## Why It Matters

Mocking a module's own internal helper functions couples the test to the implementation: refactor the internals without changing behavior, and the test breaks anyway, teaching the team to distrust red tests. Mocking only true external boundaries — HTTP calls, the filesystem, the system clock, random number generation — keeps tests validating observable behavior while leaving you free to rewrite internals. It also catches real bugs, since the code between the boundary and the assertion actually runs.

## Bad

```typescript
import { describe, expect, it, vi } from "vitest";
import * as pricing from "./pricing";
import { checkout } from "./checkout";

// Mocking an internal collaborator inside the same module boundary
vi.spyOn(pricing, "calculateSubtotal").mockReturnValue(100);

describe("checkout", () => {
  it("should return a receipt", () => {
    const receipt = checkout({ items: [] });
    expect(receipt.subtotal).toBe(100); // just echoes the mock
  });
});
```

## Good

```typescript
import { describe, expect, it, vi } from "vitest";
import { checkout } from "./checkout";
import * as httpClient from "./http-client";

describe("checkout", () => {
  it("should include tax fetched from the tax service", async () => {
    // Mock the real external boundary: an outbound HTTP call
    vi.spyOn(httpClient, "get").mockResolvedValue({ rate: 0.08 });

    const receipt = await checkout({ items: [{ price: 100 }] });

    expect(receipt.tax).toBeCloseTo(8);
    expect(receipt.total).toBeCloseTo(108);
  });
});
```

## What Counts as a Boundary

| Mock this | Don't mock this |
|---|---|
| `fetch` / HTTP client | A private helper function in the same file |
| Filesystem (`fs.readFile`) | A pure calculation the module exports for reuse |
| `Date.now()` / system clock | Another class in the same module under test |
| Database driver / ORM client | The class under test's own methods |
| Third-party SDKs | Well-tested internal libraries you already trust |

If you find yourself mocking three internal collaborators to test one function, the function's dependencies are probably better injected and tested through the real objects, or the function is doing too much.

## See Also

- [test-test-doubles](test-test-doubles.md) - Choose the right test double: stub, spy, mock, or fake
- [test-fake-timers](test-fake-timers.md) - Use fake timers to test time-dependent code deterministically
- [test-integration-vs-unit](test-integration-vs-unit.md) - Balance the test pyramid between unit and integration tests
