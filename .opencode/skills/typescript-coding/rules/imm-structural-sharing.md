# imm-structural-sharing

> Use structural sharing so immutable updates don't copy untouched subtrees

## Why It Matters

Naively deep-cloning an entire state tree on every update (`JSON.parse(JSON.stringify(state))`, or spreading at every nesting level) is both slow and wasteful: it allocates new objects for branches that didn't change, defeats reference-equality memoization on those untouched branches, and can silently drop non-JSON-safe values (`Date`, `Map`, `undefined`, functions). Structural sharing updates only the path from the root to the changed node, reusing every other reference unchanged — so sibling branches keep `===` identity and consumers that memoize on reference equality correctly skip re-computation.

## Bad

```typescript
interface AppState {
  user: { id: string; name: string };
  settings: { theme: string; locale: string };
  posts: { id: string; title: string }[];
}

function setTheme(state: AppState, theme: string): AppState {
  // Deep clones EVERYTHING, including `user` and `posts`, which didn't change.
  const next: AppState = JSON.parse(JSON.stringify(state));
  next.settings.theme = theme;
  return next;
}

const s1 = setTheme(state, "dark");
s1.user === state.user; // false — new reference even though nothing changed
// Any component memoized on `user` reference now re-renders for no reason.
```

## Good

```typescript
interface AppState {
  user: { id: string; name: string };
  settings: { theme: string; locale: string };
  posts: { id: string; title: string }[];
}

function setTheme(state: AppState, theme: string): AppState {
  return {
    ...state,
    settings: { ...state.settings, theme },
  };
}

const s1 = setTheme(state, "dark");
s1.user === state.user;     // true — untouched subtree, same reference
s1.posts === state.posts;   // true — untouched subtree, same reference
s1.settings === state.settings; // false — this is the branch that changed
```

Only the path `state -> settings` gets new objects; `user` and `posts` are reused as-is.

## Structural Sharing With a Library (Immer)

Manually spreading at every level of a deeply nested tree gets unwieldy fast. [Immer](https://immerjs.github.io/immer/) lets you write mutation-style code against a draft, and produces a structurally-shared immutable result behind the scenes:

```typescript
import { produce } from "immer";

const next = produce(state, (draft) => {
  draft.settings.theme = "dark"; // looks like mutation, isn't
});

next.user === state.user; // true — Immer only touches the path you wrote to
```

## See Also

- [imm-spread-not-mutate](imm-spread-not-mutate.md) - Create updated copies with spread/rest instead of mutating in place
- [imm-immutable-collections](imm-immutable-collections.md) - Consider a persistent/immutable collection library for hot mutation-heavy state paths
- [perf-avoid-deep-clone](perf-avoid-deep-clone.md) - Avoid deep-cloning data when a shallow copy or structural sharing suffices
