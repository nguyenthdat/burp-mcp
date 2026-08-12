# imm-avoid-array-mutation

> Avoid mutating array methods (`push`, `splice`, `sort`) on shared/shared-reference arrays

## Why It Matters

`push`, `pop`, `splice`, `sort`, `reverse`, and `shift`/`unshift` all mutate the array in place and return either the mutated array, a removed element, or a length — not a new array. When the array reference is shared (passed as a prop, stored in state, cached, held by another module), an in-place mutation is invisible to reference-equality checks used by React, Redux, memoization, and change-detection systems, and it silently corrupts data for every other holder of that reference.

## Bad

```typescript
function addItem(cart: string[], item: string): string[] {
  cart.push(item); // mutates the caller's array
  return cart;
}

function sortedByName<T extends { name: string }>(items: T[]): T[] {
  return items.sort((a, b) => a.name.localeCompare(b.name)); // sort() mutates in place!
}

const state = { items: ["a", "b"] };
const next = addItem(state.items, "c");
next === state.items; // true — React/Redux won't detect this as a change
```

## Good

```typescript
function addItem(cart: readonly string[], item: string): string[] {
  return [...cart, item];
}

function sortedByName<T extends { name: string }>(items: readonly T[]): T[] {
  return [...items].sort((a, b) => a.name.localeCompare(b.name));
}

const state = { items: ["a", "b"] };
const next = addItem(state.items, "c");
next === state.items; // false — a genuinely new array, safe for shallow-equality checks
```

## Mutating Method to Non-Mutating Equivalent

| Mutating | Non-mutating replacement |
|---|---|
| `arr.push(x)` | `[...arr, x]` |
| `arr.pop()` | `arr.slice(0, -1)` |
| `arr.shift()` | `arr.slice(1)` |
| `arr.unshift(x)` | `[x, ...arr]` |
| `arr.splice(i, 1)` | `arr.filter((_, idx) => idx !== i)` |
| `arr.sort(cmp)` | `[...arr].sort(cmp)` or `arr.toSorted(cmp)` (ES2023) |
| `arr.reverse()` | `[...arr].reverse()` or `arr.toReversed()` (ES2023) |
| `arr.splice(i, 0, x)` | `arr.toSpliced(i, 0, x)` (ES2023) |

## ES2023 Non-Mutating Array Methods

Modern runtimes (Node 20+, all current evergreen browsers) ship `toSorted`, `toReversed`, `toSpliced`, and `with` — non-mutating siblings of the classic mutating methods. Prefer these over the manual spread pattern when your target runtime supports them; they read as clearly non-mutating and avoid an intermediate copy step.

```typescript
const next = items.toSorted((a, b) => a.value - b.value);
const updatedAt = items.with(2, replacement); // replace index 2 immutably
```

## See Also

- [imm-spread-not-mutate](imm-spread-not-mutate.md) - Create updated copies with spread/rest instead of mutating in place
- [type-readonly-arrays](type-readonly-arrays.md) - Prefer `ReadonlyArray<T>` for array parameters you don't intend to mutate
- [imm-avoid-param-mutation](imm-avoid-param-mutation.md) - Never mutate a function's input parameters
