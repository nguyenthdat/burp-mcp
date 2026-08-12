# anti-deeply-nested-ternary

> Don't nest ternary expressions deeply

## Why It Matters

A single ternary (`cond ? a : b`) is a clear, idiomatic way to express a simple either/or value. Nesting ternaries inside one another (`a ? x : b ? y : c ? z : w`) turns that clarity into a puzzle: the reader has to mentally track which condition pairs with which branch, indentation (if any) doesn't reflect actual precedence the way it would in an `if`/`else if` chain, and a misplaced parenthesis or a swapped branch is easy to introduce and hard to spot in review. Once there are more than two possible outcomes, an `if`/`else if` chain, a lookup table, or a `switch` communicates the same logic far more legibly, with each condition and its result on its own line.

## Bad

```typescript
function getShippingLabel(status: string) {
  return status === 'pending'
    ? 'Awaiting Shipment'
    : status === 'shipped'
    ? 'In Transit'
    : status === 'delivered'
    ? 'Delivered'
    : status === 'returned'
    ? 'Returned to Sender'
    : 'Unknown Status'; // which condition does this belong to? easy to lose track
}
```

## Good

```typescript
function getShippingLabel(status: OrderStatus): string {
  switch (status) {
    case 'pending':
      return 'Awaiting Shipment';
    case 'shipped':
      return 'In Transit';
    case 'delivered':
      return 'Delivered';
    case 'returned':
      return 'Returned to Sender';
    default:
      return 'Unknown Status';
  }
}

// Or, for simple key -> value mappings, a lookup object is even more concise
const SHIPPING_LABELS: Record<OrderStatus, string> = {
  pending: 'Awaiting Shipment',
  shipped: 'In Transit',
  delivered: 'Delivered',
  returned: 'Returned to Sender',
};

function getShippingLabel(status: OrderStatus): string {
  return SHIPPING_LABELS[status];
}
```

## When a Nested Ternary Is Fine

A single level of nesting used to express a genuinely small decision tree, formatted clearly, is sometimes acceptable — but even then, prefer breaking it onto separate, well-indented lines rather than a single dense line:

```typescript
const badge = score >= 90
  ? 'gold'
  : score >= 70
    ? 'silver'
    : 'bronze'; // exactly one level of nesting, clearly indented — borderline acceptable
```

Enforce a nesting limit with `no-nested-ternary` (ESLint) if the team prefers zero tolerance instead of judgment calls.

## See Also

- [type-exhaustive-switch](type-exhaustive-switch.md) - Make `switch` statements exhaustive over union types
- [fn-early-return](fn-early-return.md) - Use early returns to avoid deep nesting
- [type-discriminated-union](type-discriminated-union.md) - Model variant data with discriminated unions, not flags or strings
