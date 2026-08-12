# fn-pure-functions

> Prefer pure functions with no hidden side effects

## Why It Matters

A pure function's output depends only on its inputs, and calling it changes nothing outside itself — no writes to shared state, no I/O, no mutation of arguments. Pure functions are trivial to test (no mocking, no setup/teardown), safe to memoize, safe to call in any order or in parallel, and easy to reason about because reading the signature tells you everything the function can do. Impure functions with hidden dependencies on outer state or hidden side effects create bugs that only reproduce under specific call orders or timing, which are notoriously hard to track down.

## Bad

```typescript
let taxRate = 0.08;
let lastInvoiceTotal = 0;

function calculateTotal(price: number, quantity: number): number {
  lastInvoiceTotal = price * quantity * (1 + taxRate); // hidden write to outer state
  console.log(`Invoice total: ${lastInvoiceTotal}`); // hidden side effect (I/O)
  return lastInvoiceTotal;
}

// Same inputs, different result depending on when taxRate last changed elsewhere:
calculateTotal(100, 2);
taxRate = 0.1;
calculateTotal(100, 2); // different output for identical arguments
```

## Good

```typescript
interface TaxContext {
  taxRate: number;
}

function calculateTotal(price: number, quantity: number, ctx: TaxContext): number {
  return price * quantity * (1 + ctx.taxRate);
}

// Same inputs always produce the same output — safe to test, cache, and reorder.
calculateTotal(100, 2, { taxRate: 0.08 }); // 216
calculateTotal(100, 2, { taxRate: 0.08 }); // 216, always

// Logging is a separate, explicit concern at the call site:
const total = calculateTotal(100, 2, { taxRate: 0.08 });
console.log(`Invoice total: ${total}`);
```

## Recognizing Impurity

A function is impure if it does any of the following:
- Reads or writes a variable outside its own scope (module-level `let`, a class field, a global).
- Mutates any of its arguments (see `imm-avoid-param-mutation`).
- Performs I/O: network calls, file access, `console.log`, `Date.now()`, `Math.random()`.
- Throws for some inputs but not others in a way not reflected in its return type (an unexpected exception is itself a hidden "output").

## Isolating the Impure Edges

You cannot make an entire application pure — it has to talk to the network and the clock eventually. The practical goal is to push impurity to the edges (I/O, framework boundary, `main`) and keep the core business logic pure:

```typescript
// Impure edge: reads the current time
function getCurrentTaxRate(): number {
  return isHolidaySeason(new Date()) ? 0.05 : 0.08;
}

// Pure core: everything it needs comes in as an argument
function calculateTotal(price: number, quantity: number, taxRate: number): number {
  return price * quantity * (1 + taxRate);
}

// Composition happens at the boundary:
const total = calculateTotal(100, 2, getCurrentTaxRate());
```

## See Also

- [imm-avoid-param-mutation](imm-avoid-param-mutation.md) - Never mutate a function's input parameters
- [fn-composition-over-inheritance](fn-composition-over-inheritance.md) - Compose small functions instead of building class inheritance hierarchies
- [anti-global-mutable-state](anti-global-mutable-state.md) - Avoid module-level mutable state shared across call sites
