# imm-object-freeze-runtime

> Use `Object.freeze` when you need a runtime immutability guarantee, not just a compile-time one

## Why It Matters

TypeScript's `readonly` and `as const` are erased at compile time — they produce zero JavaScript and cannot stop a caller who bypasses the type checker (via `any`, a `.js` consumer, JSON deserialization, or a third-party library that ignores your types). If a shared constant, config object, or module-level singleton must never be mutated regardless of how it's accessed, `Object.freeze` enforces that at runtime: attempting to write to a frozen object silently no-ops in sloppy mode or throws a `TypeError` in strict mode/ESM.

## Bad

```typescript
export const DEFAULT_CONFIG: Readonly<{ timeout: number; retries: number }> = {
  timeout: 5000,
  retries: 3,
};

// readonly is compile-time only — this still runs and mutates the shared object
(DEFAULT_CONFIG as any).timeout = 0;

function poison(cfg: any) {
  cfg.retries = 999; // no type error inside an `any`-typed function
}
poison(DEFAULT_CONFIG);
```

## Good

```typescript
export const DEFAULT_CONFIG = Object.freeze({
  timeout: 5000,
  retries: 3,
});

function poison(cfg: any) {
  cfg.retries = 999; // throws TypeError in strict mode / ESM modules
}
poison(DEFAULT_CONFIG); // Uncaught TypeError: Cannot assign to read only property
```

## Deep Freeze for Nested Objects

`Object.freeze` is shallow — nested objects remain mutable unless you freeze recursively:

```typescript
function deepFreeze<T>(value: T): Readonly<T> {
  if (value !== null && (typeof value === "object" || typeof value === "function")) {
    Object.getOwnPropertyNames(value).forEach((key) => {
      deepFreeze((value as Record<string, unknown>)[key]);
    });
    Object.freeze(value);
  }
  return value;
}

const settings = deepFreeze({
  server: { host: "localhost", port: 8080 },
  features: ["logging", "metrics"],
});

settings.server.port = 9090; // throws — nested object is also frozen
```

## When This Is Worth The Runtime Cost

- Shared constants/config exported from a module and consumed by code you don't control (plugins, dynamic `require`, untyped JS callers).
- Values passed across a serialization boundary (worker `postMessage`, IPC) where mutation would cause silent divergence.
- Redux-style state objects during development, to catch accidental mutation early (many teams only freeze in non-production builds to avoid the perf cost).

Avoid freezing in hot paths — the freeze call and the subsequent write-checks have a real, measurable cost in tight loops.

## See Also

- [imm-as-const-literal](imm-as-const-literal.md) - Freeze literal object/array structures with `as const`
- [imm-deep-immutability-types](imm-deep-immutability-types.md) - Use a deep-readonly utility type for nested immutable state trees
- [type-readonly-arrays](type-readonly-arrays.md) - Prefer `ReadonlyArray<T>` for array parameters you don't intend to mutate
