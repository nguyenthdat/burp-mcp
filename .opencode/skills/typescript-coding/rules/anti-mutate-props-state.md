# anti-mutate-props-state

> Don't mutate props or shared state objects directly

## Why It Matters

When a function mutates an object it was merely handed — a React prop, a Redux/Zustand state slice, an object shared between modules — every other holder of a reference to that object sees the change too, often at a time and in a way they didn't expect. This breaks referential-equality checks that memoization (`React.memo`, `useMemo`, selector libraries) depend on to skip unnecessary re-renders, makes bugs nondeterministic based on call order, and makes "what changed and who changed it" nearly impossible to trace in a debugger since the mutation leaves no history. Treating props and shared state as read-only, and producing new objects for any change, keeps state changes explicit, traceable, and compatible with the equality checks the rest of the ecosystem relies on.

## Bad

```typescript
interface CartProps {
  items: CartItem[];
}

function addDiscount(props: CartProps, discount: number) {
  // Mutates the caller's array in place — a shared reference, unexpected side effect
  props.items.forEach((item) => {
    item.price = item.price * (1 - discount);
  });
}

// React: this props.items reference is unchanged, so memo() won't re-render
function ProductList({ items }: { items: Product[] }) {
  items.push(newProduct); // mutating a prop directly
  return <ul>{items.map((i) => <li key={i.id}>{i.name}</li>)}</ul>;
}
```

## Good

```typescript
interface CartProps {
  items: readonly CartItem[];
}

function applyDiscount(items: readonly CartItem[], discount: number): CartItem[] {
  return items.map((item) => ({ ...item, price: item.price * (1 - discount) }));
}

function ProductList({ items }: { items: readonly Product[] }) {
  const withNew = [...items, newProduct]; // new array, new reference
  return <ul>{withNew.map((i) => <li key={i.id}>{i.name}</li>)}</ul>;
}
```

## Enforcing Immutability at the Type Level

```typescript
// `readonly` on props/parameters makes accidental mutation a compile error
function render(props: Readonly<{ items: readonly Item[] }>) {
  props.items.push(x); // Error: Property 'push' does not exist on type 'readonly Item[]'
}
```

For deeply nested state, use a structural-sharing library (Immer) so "mutate-looking" code produces immutable updates under the hood:

```typescript
import { produce } from 'immer';

const next = produce(state, (draft) => {
  draft.items[0].price *= 0.9; // looks like mutation, produces a new object
});
```

## See Also

- [imm-avoid-param-mutation](imm-avoid-param-mutation.md) - Don't mutate function parameters
- [imm-readonly-class-fields](imm-readonly-class-fields.md) - Mark class fields `readonly` when they shouldn't change after construction
- [imm-structural-sharing](imm-structural-sharing.md) - Use structural sharing for efficient immutable updates
