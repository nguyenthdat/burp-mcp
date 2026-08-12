# imm-avoid-param-mutation

> Never mutate a function's input parameters

## Why It Matters

When a function mutates one of its parameters, the caller's data changes as a side effect of a call that looked like it should just compute and return a value. This breaks the basic contract a reader assumes when calling `f(x)`: that `x` is unchanged afterward unless the function's name or docs say otherwise. Parameter mutation is one of the most common sources of "spooky action at a distance" bugs — a value read earlier in a function suddenly differs later because something deep in a call chain mutated it in place.

## Bad

```typescript
interface Order {
  id: string;
  items: { sku: string; qty: number }[];
  total: number;
}

function applyDiscount(order: Order, percent: number): Order {
  order.total = order.total * (1 - percent / 100); // mutates caller's object
  return order;
}

function addLineItem(order: Order, sku: string, qty: number): Order {
  order.items.push({ sku, qty }); // mutates caller's array
  return order;
}

const original = { id: "1", items: [], total: 100 };
const discounted = applyDiscount(original, 10);
original.total; // 90 — the caller's own object silently changed
```

## Good

```typescript
interface Order {
  id: string;
  items: { sku: string; qty: number }[];
  total: number;
}

function applyDiscount(order: Order, percent: number): Order {
  return { ...order, total: order.total * (1 - percent / 100) };
}

function addLineItem(order: Order, sku: string, qty: number): Order {
  return { ...order, items: [...order.items, { sku, qty }] };
}

const original = { id: "1", items: [], total: 100 };
const discounted = applyDiscount(original, 10);
original.total; // 100 — untouched
discounted.total; // 90
```

## Guarding At The Type Level

Typing parameters as `readonly` (or `Readonly<T>`/`ReadonlyArray<T>`) turns "don't mutate this" from a convention into a compiler-enforced rule, catching accidental mutation at the call site instead of relying on code review:

```typescript
function applyDiscount(order: Readonly<Order>, percent: number): Order {
  order.total = 0; // Error: cannot assign to readonly property
  return { ...order, total: order.total * (1 - percent / 100) };
}

function addLineItem(order: Order, items: readonly { sku: string; qty: number }[]) {
  items.push({ sku: "X", qty: 1 }); // Error: push does not exist on readonly array
}
```

## Exception: Explicit In-Place APIs

Some functions are documented and named specifically to mutate (`Array.prototype.sort`, `array.splice`, a `Vector.normalizeInPlace()` performance API). That's acceptable as long as the mutation is the function's obvious, named purpose — the anti-pattern is *incidental* mutation of a parameter inside a function whose name and signature suggest a pure computation.

## See Also

- [imm-spread-not-mutate](imm-spread-not-mutate.md) - Create updated copies with spread/rest instead of mutating in place
- [fn-pure-functions](fn-pure-functions.md) - Prefer pure functions with no hidden side effects
- [type-readonly-arrays](type-readonly-arrays.md) - Prefer `ReadonlyArray<T>` for array parameters you don't intend to mutate
