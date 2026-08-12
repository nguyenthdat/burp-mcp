# test-fixture-factories

> Use factory functions to build test fixtures instead of duplicating literals

## Why It Matters

Copy-pasting a full object literal into every test that needs "a user" or "an order" means that when the shape changes — a new required field is added — every test file needs a manual find-and-replace, and it's easy to miss one and leave a stale, now-invalid fixture in place. Factory functions centralize the "default valid instance" in one place with sensible defaults, let each test override only the field it cares about, and keep tests focused on what's actually being verified instead of boilerplate setup.

## Bad

```typescript
it("should reject an expired card", () => {
  const order = {
    id: "ord_1",
    customerId: "cust_1",
    items: [{ sku: "a", price: 100, qty: 1 }],
    card: { number: "4242424242424242", expiry: "01/20", cvc: "123" },
    shippingAddress: { line1: "1 Main St", city: "NYC", zip: "10001", country: "US" },
  };
  expect(() => charge(order)).toThrow("CardExpired");
});

it("should charge a valid order", () => {
  const order = {
    id: "ord_2",
    customerId: "cust_1",
    items: [{ sku: "a", price: 100, qty: 1 }],
    card: { number: "4242424242424242", expiry: "12/30", cvc: "123" },
    shippingAddress: { line1: "1 Main St", city: "NYC", zip: "10001", country: "US" },
  };
  expect(charge(order)).toBeDefined();
});
```

## Good

```typescript
// test/factories/order.ts
import type { Order } from "../../src/order";

let counter = 0;

export function buildOrder(overrides: Partial<Order> = {}): Order {
  counter += 1;
  return {
    id: `ord_${counter}`,
    customerId: "cust_1",
    items: [{ sku: "a", price: 100, qty: 1 }],
    card: { number: "4242424242424242", expiry: "12/30", cvc: "123" },
    shippingAddress: { line1: "1 Main St", city: "NYC", zip: "10001", country: "US" },
    ...overrides,
  };
}
```

```typescript
it("should reject an expired card", () => {
  const order = buildOrder({ card: { number: "4242424242424242", expiry: "01/20", cvc: "123" } });
  expect(() => charge(order)).toThrow("CardExpired");
});

it("should charge a valid order", () => {
  expect(charge(buildOrder())).toBeDefined();
});
```

## Common Patterns

- Give each factory a unique, auto-incrementing id or a random suffix so tests running in parallel never collide on a hardcoded id.
- Compose factories for nested objects: `buildOrder({ items: [buildLineItem({ qty: 3 })] })`.
- Keep factories in `test/factories/` (or colocated per module) and export one function per domain entity, not one giant "test data" file.
- Avoid factories that read from fixtures on disk for simple unit tests — plain functions returning object literals are faster and easier to override.

## See Also

- [test-arrange-act-assert](test-arrange-act-assert.md) - Structure tests as arrange/act/assert
- [test-isolate-tests](test-isolate-tests.md) - Keep tests isolated and order-independent, with no shared mutable state
- [test-parameterized-tests](test-parameterized-tests.md) - Use parameterized/table-driven tests (`it.each`) for input/output variants
