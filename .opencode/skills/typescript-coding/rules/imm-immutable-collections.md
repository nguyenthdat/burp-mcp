# imm-immutable-collections

> Consider a persistent/immutable collection library for hot mutation-heavy state paths

## Why It Matters

Naive immutability with spread (`{ ...obj }`, `[...arr]`) copies the entire object or array on every update. For small objects that's negligible, but for large collections updated frequently (thousands of entries, updated many times per second — undo/redo stacks, collaborative editors, real-time collections), full copies become an O(n) cost per update and produce heavy GC pressure. Persistent data structures (based on structures like hash array mapped tries) give you the same "never mutate, always get a new immutable value back" API while sharing almost all of the underlying memory between versions, making updates closer to O(log n).

## Bad

```typescript
// A large, frequently-updated collection, copied fully on every change.
class DocumentStore {
  private lines: readonly string[] = [];

  insertLine(index: number, text: string) {
    // O(n) copy on every keystroke in a 50,000-line document.
    this.lines = [...this.lines.slice(0, index), text, ...this.lines.slice(index)];
  }
}
```

## Good

```typescript
import { List } from "immutable";

class DocumentStore {
  private lines = List<string>();

  insertLine(index: number, text: string) {
    // O(log n) structural update; old versions remain valid and cheap to keep around.
    this.lines = this.lines.insert(index, text);
  }

  getSnapshot(): List<string> {
    return this.lines; // safe to hand out — callers can't mutate it
  }
}
```

## Library Landscape

| Library | Notes |
|---|---|
| `immutable` (Immutable.js) | Mature, full collection suite (`List`, `Map`, `Set`, `Record`); own API, not plain arrays/objects |
| `immer` | Wraps plain objects/arrays; you write mutation-style "draft" code, get back structurally-shared immutable output — best for readability over raw performance |
| `mori` | ClojureScript-style persistent structures for JS; less common in modern TS codebases |

## When To Reach For One

- The collection is large (thousands+ entries) **and** updated frequently in a hot path (real-time collab, animation state, undo/redo history).
- Profiling has actually shown copy cost or GC churn from spread-based updates to be a bottleneck — don't add this dependency preemptively.
- You need cheap structural equality checks across many historical versions (e.g., time-travel debugging).

For most application state (forms, typical component state, REST responses), plain objects with spread or Immer are simpler, have zero API lock-in, and are fast enough. Reach for a persistent collection library only after measuring a real cost.

## See Also

- [imm-structural-sharing](imm-structural-sharing.md) - Use structural sharing so immutable updates don't copy untouched subtrees
- [perf-avoid-unnecessary-allocation](perf-avoid-unnecessary-allocation.md) - Avoid unnecessary allocation in hot paths
- [perf-avoid-deep-clone](perf-avoid-deep-clone.md) - Avoid deep-cloning data when a shallow copy or structural sharing suffices
