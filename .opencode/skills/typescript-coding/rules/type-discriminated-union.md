# type-discriminated-union

> Model variants with discriminated unions and a common tag field

## Why It Matters

Representing "one of several possible shapes" with optional fields (`{ success?: boolean; data?: T; error?: string }`) lets you construct nonsensical states, like an object that has both `data` and `error` set, or neither. A discriminated union with a shared literal tag field makes every variant mutually exclusive and lets the compiler narrow the whole object automatically based on the tag, eliminating an entire category of "field X is undefined when I expected it" bugs.

## Bad

```typescript
interface FetchState<T> {
  loading?: boolean;
  data?: T;
  error?: string;
}

function render<T>(state: FetchState<T>) {
  // Nothing stops loading, data, and error from all being set at once
  if (state.loading) {
    return "Loading...";
  }
  if (state.error) {
    return `Error: ${state.error}`;
  }
  // TypeScript can't guarantee data is defined here
  return JSON.stringify(state.data);
}
```

## Good

```typescript
type FetchState<T> =
  | { status: "loading" }
  | { status: "success"; data: T }
  | { status: "error"; error: string };

function render<T>(state: FetchState<T>): string {
  switch (state.status) {
    case "loading":
      return "Loading...";
    case "success":
      return JSON.stringify(state.data); // data is guaranteed present
    case "error":
      return `Error: ${state.error}`; // error is guaranteed present
  }
}
```

## Construction Helpers

Factory functions keep call sites from having to remember every required field per variant:

```typescript
const loading = <T>(): FetchState<T> => ({ status: "loading" });
const success = <T>(data: T): FetchState<T> => ({ status: "success", data });
const failure = <T>(error: string): FetchState<T> => ({ status: "error", error });
```

## Choosing a Tag Field

| Convention | Example | Notes |
|---|---|---|
| `status` | `"loading" \| "success" \| "error"` | Good for async/request state |
| `kind` / `type` | `"circle" \| "square"` | Good for domain entities, AST nodes |
| `_tag` | `"Some" \| "None"` | Common in functional-style libraries (fp-ts, effect) |

Pick one convention per codebase and stick with it — mixing `kind` and `type` across modules makes union types harder to scan.

## See Also

- [type-exhaustive-switch](type-exhaustive-switch.md) - Enforce exhaustiveness checks with a never assertion
- [type-narrow-guards](type-narrow-guards.md) - Use user-defined type guards to narrow union types safely
- [err-result-pattern](err-result-pattern.md) - Use a Result-like return type for expected, recoverable failures
