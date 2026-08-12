# anti-global-mutable-state

> Don't rely on global mutable state

## Why It Matters

A module-level `let` variable, a singleton with mutable fields, or anything attached to `globalThis` is implicitly shared by every part of the program that imports the module — any code, anywhere, can read or change it, and there's no way to trace who did so from the type system or from a function's signature alone. This makes behavior depend on *execution order* (which test ran first, which request handler ran first) rather than on explicit inputs, which is a primary cause of flaky tests, "works locally but not in CI" bugs, and race conditions in concurrent request handling (a global counter mutated by two simultaneous requests without synchronization). Passing state explicitly — as a parameter, a class instance, or through dependency injection — makes every function's true inputs visible in its signature and makes state changes traceable.

## Bad

```typescript
// cache.ts
let cache: Record<string, unknown> = {}; // module-level mutable global

export function setCache(key: string, value: unknown) {
  cache[key] = value; // any importer of this module can mutate shared state
}

export function getCache(key: string) {
  return cache[key];
}

// Two concurrent requests both mutate the same object with no isolation,
// and tests that run in the same process leak state into each other
```

## Good

```typescript
// cache.ts
export class Cache {
  private store = new Map<string, unknown>();

  set(key: string, value: unknown) {
    this.store.set(key, value);
  }

  get(key: string) {
    return this.store.get(key);
  }
}

// Each request (or each test) gets its own instance — no shared, hidden state
function createRequestContext() {
  return { cache: new Cache() };
}
```

```typescript
// Dependency injection makes the dependency explicit in the function signature
function handleRequest(req: Request, cache: Cache) {
  const cached = cache.get(req.url);
  // ...
}
```

## When Module-Level State Is Acceptable

A module-level constant that is never reassigned (`const config = loadConfig()`, computed once at startup) is not the problem — the issue is specifically *mutable* state shared across otherwise-independent call paths. A connection pool or a logger instance created once at startup and never mutated afterward is a reasonable module-level singleton; a mutable cache, counter, or flag that changes based on runtime events is not.

## See Also

- [imm-prefer-const](imm-prefer-const.md) - Prefer `const` bindings for values that never change
- [fn-pure-functions](fn-pure-functions.md) - Prefer pure functions with no hidden side effects
- [test-isolate-tests](test-isolate-tests.md) - Keep tests isolated from each other
