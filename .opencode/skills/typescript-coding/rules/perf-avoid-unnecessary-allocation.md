# perf-avoid-unnecessary-allocation

> Avoid allocating objects/arrays inside hot loops

## Why It Matters

Allocating a new object, array, or closure on every iteration of a hot loop puts constant pressure on the garbage collector, which can introduce pause times that show up as jank in UI code or throughput drops in server code processing high volumes of requests. Most of the time these allocations are wasted work: the object is used once and discarded, and hoisting it out of the loop (or reusing a buffer) eliminates the churn entirely without changing the loop's observable behavior.

## Bad

```typescript
function sumDistances(points: Array<{ x: number; y: number }>, origin: { x: number; y: number }): number {
  let total = 0;
  for (const p of points) {
    // Allocates a new object and a new closure every iteration
    const delta = { dx: p.x - origin.x, dy: p.y - origin.y };
    const distance = Math.sqrt(delta.dx ** 2 + delta.dy ** 2);
    total += distance;
  }
  return total;
}
```

## Good

```typescript
function sumDistances(points: Array<{ x: number; y: number }>, origin: { x: number; y: number }): number {
  let total = 0;
  for (const p of points) {
    // Plain numbers, no per-iteration object allocation
    const dx = p.x - origin.x;
    const dy = p.y - origin.y;
    total += Math.sqrt(dx * dx + dy * dy);
  }
  return total;
}
```

## Common Patterns

- Hoist object/array literals that don't change per iteration outside the loop entirely.
- Avoid creating a new closure (`.map(x => ...)`, arrow functions capturing loop variables) inside a loop that runs millions of times; define the function once outside the loop if it doesn't need per-iteration state.
- For high-throughput binary/numeric processing, reuse a pre-allocated `TypedArray` or buffer instead of pushing into a plain array that has to grow and reallocate repeatedly.
- This rule applies to genuinely hot paths — request handlers processing thousands of items per second, tight rendering loops, streaming parsers — not general application code, where the allocation cost is invisible next to I/O latency. Profile first (see `perf-avoid-premature-optimize`) before restructuring readable code for this reason.

## See Also

- [perf-avoid-premature-optimize](perf-avoid-premature-optimize.md) - Profile before optimizing
- [perf-avoid-deep-clone](perf-avoid-deep-clone.md) - Avoid deep cloning when structural sharing or shallow copies suffice
- [imm-structural-sharing](imm-structural-sharing.md) - reuse unchanged parts of a data structure instead of reallocating
