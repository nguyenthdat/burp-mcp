# test-descriptive-names

> Name tests descriptively: "should X when Y"

## Why It Matters

A test suite is a specification of behavior, and its names are the table of contents. When a name like `test1` or `handles edge case` fails in CI, nobody can tell what broke without opening the file and reading the assertions. Descriptive names ("should throw InvalidEmailError when the address has no @") let a failing CI run communicate the regression directly in the test report, and they double as living documentation for new contributors skimming `describe` blocks.

## Bad

```typescript
describe("UserService", () => {
  it("works", () => {
    const service = new UserService(fakeRepo);
    expect(() => service.register("not-an-email", "pw")).toThrow();
  });

  it("test 2", () => {
    const service = new UserService(fakeRepo);
    expect(service.register("a@b.com", "pw")).toBeDefined();
  });
});
```

## Good

```typescript
describe("UserService.register", () => {
  it("should throw InvalidEmailError when the email has no @ symbol", () => {
    const service = new UserService(fakeRepo);
    expect(() => service.register("not-an-email", "pw")).toThrow(InvalidEmailError);
  });

  it("should return a new User when given a valid email and password", () => {
    const service = new UserService(fakeRepo);
    expect(service.register("a@b.com", "pw")).toEqual(expect.objectContaining({ email: "a@b.com" }));
  });
});
```

## Naming Template

| Part | Example |
|---|---|
| Subject | `UserService.register` |
| Behavior | `should throw InvalidEmailError` |
| Condition | `when the email has no @ symbol` |

Compose these into `it("should <behavior> when <condition>", ...)`. For pure functions with no branching condition, drop the "when" clause: `it("should return the sum of two positive numbers", ...)`.

Nest `describe` blocks by unit under test (class, module, or function) so the full test name reads as a sentence: `UserService.register > should throw InvalidEmailError when the email has no @ symbol`.

## See Also

- [test-arrange-act-assert](test-arrange-act-assert.md) - Structure tests as arrange/act/assert
- [test-parameterized-tests](test-parameterized-tests.md) - Use parameterized/table-driven tests (`it.each`) for input/output variants
- [name-verb-noun-functions](name-verb-noun-functions.md) - related naming convention for production code
