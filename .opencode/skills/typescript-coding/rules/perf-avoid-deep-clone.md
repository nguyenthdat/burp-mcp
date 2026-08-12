# perf-avoid-deep-clone

> Avoid deep cloning when structural sharing or shallow copies suffice

## Why It Matters

`structuredClone()` or a hand-rolled deep clone walks and duplicates every nested object and array, which is expensive for large or deeply nested data and creates garbage the collector then has to reclaim — often for no real benefit, since most updates only touch one or two levels of a structure. Structural sharing (reusing references to the parts of a structure that didn't change) or a shallow copy at just the mutated level achieves the same immutability guarantee at a fraction of the cost.

## Bad

```typescript
interface State {
  user: { id: string; name: string; preferences: { theme: string; locale: string } };
  cart: { items: CartItem[] };
}

function updateTheme(state: State, theme: string): State {
  // Deep-clones the entire state tree just to change one leaf field
  const next = structuredClone(state);
  next.user.preferences.theme = theme;
  return next;
}
```

## Good

```typescript
function updateTheme(state: State, theme: string): State {
  // Structural sharing: only the path to the changed field is copied;
  // `cart` and unrelated parts of `user` keep their original references.
  return {
    ...state,
    user: {
      ...state.user,
      preferences: { ...state.user.preferences, theme },
    },
  };
}
```

## When Deep Cloning Is Actually Needed

| Situation | Recommendation |
|---|---|
| Updating one nested field in application state | Structural sharing via spread (shown above) |
| Passing data across a `postMessage`/worker boundary | `structuredClone()` is correct and necessary — the receiving context needs its own copy |
| Snapshotting state for an undo stack | Structural sharing usually suffices if consumers treat past states as read-only |
| Deep-copying to satisfy a leaky external API that mutates its input | Deep clone is a legitimate defensive measure, but consider fixing the leaky API's contract instead |

Libraries like Immer let you write code that looks like direct mutation while producing a structurally shared new object automatically, which is often easier to get right than hand-written nested spreads for deeply nested state.

## See Also

- [imm-structural-sharing](imm-structural-sharing.md) - the underlying technique this rule recommends
- [imm-spread-not-mutate](imm-spread-not-mutate.md) - use spread for shallow, non-mutating updates
- [perf-avoid-unnecessary-allocation](perf-avoid-unnecessary-allocation.md) - Avoid allocating objects/arrays inside hot loops
