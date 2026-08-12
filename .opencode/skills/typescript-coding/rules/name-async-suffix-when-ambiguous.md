# name-async-suffix-when-ambiguous

> Suffix an async function's name when a sync counterpart exists with the same base name

## Why It Matters

When a codebase has both a synchronous and an asynchronous way to do the same conceptual operation (`readFile`/`readFileSync` style APIs, a cached in-memory getter versus a network-backed fetch), giving both the exact same base name forces every caller to check the signature or return type to know which one they're calling — and an accidental call to the wrong one either blocks the event loop (sync where async was expected) or silently introduces an unawaited `Promise` (async where sync was assumed). An explicit suffix removes that ambiguity at the call site, before a reader even needs to check the return type.

## Bad

```typescript
class UserStore {
  private cache = new Map<string, User>();

  // Two methods, same name pattern, silently different behavior:
  getUser(id: string): User | undefined {
    return this.cache.get(id); // synchronous, cache-only
  }

  async getUser2(id: string): Promise<User> {
    // fetches from network if not cached — but the name gives no hint
    return fetch(`/api/users/${id}`).then((r) => r.json());
  }
}

// Caller has no signal from the name alone which one they're invoking:
const user = getUser(id); // is this a Promise or a User? Have to check the signature.
```

## Good

```typescript
class UserStore {
  private cache = new Map<string, User>();

  getUserFromCache(id: string): User | undefined {
    return this.cache.get(id);
  }

  async fetchUser(id: string): Promise<User> {
    return fetch(`/api/users/${id}`).then((r) => r.json());
  }
}

// The names alone tell you which is synchronous and which returns a Promise.
const cached = userStore.getUserFromCache(id);
const fresh = await userStore.fetchUser(id);
```

## Node's Own Convention: `Sync` Suffix

Node's built-in APIs establish the inverse convention — the async version keeps the plain name, and the *synchronous* variant is explicitly suffixed, since async is the expected default in Node:

```typescript
import { readFile, readFileSync } from "node:fs";

readFile("./config.json", "utf-8", (err, data) => { /* ... */ }); // async, callback-based
const raw = readFileSync("./config.json", "utf-8");                // sync, blocks the event loop
```

Follow whichever direction matches your codebase's dominant style consistently — either "async is unmarked, sync gets `Sync`" (matching Node's fs module) or "sync is unmarked, async gets a verb like `fetch`/`load`/an explicit suffix" — but don't let the same base name silently mean both in different places.

## When No Suffix Is Needed

If only one version (sync or async) of an operation exists anywhere in the codebase, there's no ambiguity to resolve — adding an `Async` suffix to every async function regardless of whether a sync counterpart exists is unnecessary noise (`async function fetchUserAsync()` when there's no `fetchUser` at all).

## See Also

- [name-verb-noun-functions](name-verb-noun-functions.md) - Name functions with a leading verb describing the action they perform
- [node-avoid-sync-fs](node-avoid-sync-fs.md) - Avoid synchronous filesystem calls that block the event loop
- [async-await-over-then](async-await-over-then.md) - Prefer `async`/`await` over chained `.then()` for readability
