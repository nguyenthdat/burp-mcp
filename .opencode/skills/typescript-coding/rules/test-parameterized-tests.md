# test-parameterized-tests

> Use parameterized/table-driven tests (`it.each`) for input/output variants

## Why It Matters

Copy-pasting the same test body five times with slightly different inputs bloats the file and, worse, tends to drift: one copy gets a bug fix or an extra assertion that the others don't. `it.each`/`test.each` turns a table of cases into a single, exhaustively described test, making it trivial to add a new edge case as one more row and guaranteeing every case runs through identical assertion logic. It also produces one clearly named failure per case instead of a single monolithic test that fails ambiguously.

## Bad

```typescript
import { expect, it } from "vitest";
import { classifyAge } from "./classify-age";

it("should classify 5 as a child", () => {
  expect(classifyAge(5)).toBe("child");
});
it("should classify 15 as a teen", () => {
  expect(classifyAge(15)).toBe("teen");
});
it("should classify 30 as an adult", () => {
  expect(classifyAge(30)).toBe("adult");
});
it("should classify 70 as a senior", () => {
  expect(classifyAge(70)).toBe("senior");
});
```

## Good

```typescript
import { describe, expect, it } from "vitest";
import { classifyAge } from "./classify-age";

describe.each([
  { age: 5, expected: "child" },
  { age: 15, expected: "teen" },
  { age: 30, expected: "adult" },
  { age: 70, expected: "senior" },
])("classifyAge($age)", ({ age, expected }) => {
  it(`should return "${expected}"`, () => {
    expect(classifyAge(age)).toBe(expected);
  });
});
```

## Common Patterns

- Use an array of plain objects (not tuples) for cases with more than two fields — named properties (`{ age, expected }`) read far better in failure output than positional `[5, "child"]`.
- Keep one assertion focus per parameterized test; if different cases need different assertions, they're not really the same test and shouldn't share a table.
- Combine with `test.each` template-literal tables for simple two-column cases: `it.each\`a | b | expected\n${1} | ${2} | ${3}\`("$a + $b = $expected", ({ a, b, expected }) => ...)`.
- Parameterized tests are ideal for validation rules, boundary values, and pure-function input/output mappings; they're a poor fit for tests with meaningfully different setup per case.

## See Also

- [test-descriptive-names](test-descriptive-names.md) - Name tests descriptively: "should X when Y"
- [test-fixture-factories](test-fixture-factories.md) - Use factory functions to build test fixtures instead of duplicating literals
- [test-coverage-meaningful](test-coverage-meaningful.md) - Target meaningful coverage of behavior, not a 100% coverage vanity metric
