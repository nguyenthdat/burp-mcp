# test-coverage-meaningful

> Target meaningful coverage of behavior, not a 100% coverage vanity metric

## Why It Matters

Coverage percentage measures which lines executed during a test run, not whether the test actually asserted anything meaningful about them. Chasing 100% produces tests that call a function and assert nothing, or assert on trivial output just to paint a line green, which burns engineering time and gives false confidence — a covered branch with a weak assertion is barely better than an untested one. Coverage is a diagnostic tool for finding gaps, not a target to optimize; the goal is confidence that behavior is correct, especially for business logic, edge cases, and error paths.

## Bad

```typescript
import { expect, it } from "vitest";
import { calculateShipping } from "./shipping";

// Covers the line but asserts nothing meaningful — written purely to hit 100%
it("calculateShipping runs", () => {
  const result = calculateShipping({ weightKg: 5, country: "US" });
  expect(result).toBeDefined();
});
```

## Good

```typescript
import { expect, it } from "vitest";
import { calculateShipping } from "./shipping";

it("should charge the flat domestic rate for shipments under 10kg", () => {
  expect(calculateShipping({ weightKg: 5, country: "US" })).toBe(4.99);
});

it("should throw UnsupportedCountryError for an unlisted country", () => {
  expect(() => calculateShipping({ weightKg: 5, country: "XX" })).toThrow("UnsupportedCountryError");
});

it("should apply the overweight surcharge at exactly the 10kg boundary", () => {
  expect(calculateShipping({ weightKg: 10, country: "US" })).toBe(9.99);
});
```

## Where To Focus Coverage

| Priority | What to cover |
|---|---|
| High | Business rules, pricing/financial logic, auth and authorization checks |
| High | Error paths and validation failures, not just the happy path |
| High | Boundary conditions (empty arrays, zero, max values, off-by-one edges) |
| Medium | Integration points between modules |
| Low | Simple pass-through getters, framework glue code, generated code |

Use coverage reports (`vitest --coverage`) to find *unexpected* gaps — an untested branch in payment logic is a signal; an uncovered trivial re-export is not. Set thresholds (e.g. 80% lines, 100% on `src/billing/**`) tied to risk, not a blanket 100%.

## See Also

- [test-integration-vs-unit](test-integration-vs-unit.md) - Balance the test pyramid between unit and integration tests
- [test-vitest-jest-setup](test-vitest-jest-setup.md) - Follow standard Vitest/Jest project conventions for config and structure
- [test-parameterized-tests](test-parameterized-tests.md) - Use parameterized/table-driven tests (`it.each`) for input/output variants
