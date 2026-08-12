# test-arrange-act-assert

> Structure tests as arrange/act/assert

## Why It Matters

Tests that interleave setup, invocation, and verification are hard to scan and harder to debug when they fail — a reader can't tell at a glance what the test actually exercises. The arrange/act/assert (AAA) shape gives every test a predictable structure: build the inputs, perform the one action under test, then check the outcome. This makes diffs smaller when a test changes, makes failures easier to localize, and makes it obvious when a test is doing too much (multiple "acts" is a signal to split the test).

## Bad

```typescript
test("applies discount", () => {
  const cart = new Cart();
  cart.addItem({ id: "sku-1", price: 100 });
  expect(cart.items).toHaveLength(1);
  cart.applyDiscount(0.1);
  const total = cart.total();
  cart.addItem({ id: "sku-2", price: 50 });
  expect(total).toBe(90);
});
```

## Good

```typescript
test("should apply a 10% discount to the cart total", () => {
  // Arrange
  const cart = new Cart();
  cart.addItem({ id: "sku-1", price: 100 });

  // Act
  cart.applyDiscount(0.1);
  const total = cart.total();

  // Assert
  expect(total).toBe(90);
});
```

## Common Patterns

- Keep the "Act" section to a single statement (or a single logical operation) whenever possible — if you need two acts, you probably need two tests.
- Extract repeated "Arrange" blocks into fixture factories (see `test-fixture-factories`) rather than copy-pasting setup.
- The AAA comments are optional scaffolding for readers; a blank line between each section is often enough once the team is used to the convention.
- For asynchronous code, `await` belongs in the "Act" section: `const result = await service.process(input);`.

## See Also

- [test-fixture-factories](test-fixture-factories.md) - Use factory functions to build test fixtures instead of duplicating literals
- [test-descriptive-names](test-descriptive-names.md) - Name tests descriptively: "should X when Y"
- [test-isolate-tests](test-isolate-tests.md) - Keep tests isolated and order-independent, with no shared mutable state
