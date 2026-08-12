# api-avoid-optional-overuse

> Avoid excessive optional properties; model valid states as required unions instead

## Why It Matters

Marking many properties optional (`?`) creates a type that permits combinations that don't correspond to any real, valid state of your domain — e.g. a "loaded" state where the data is optional and the error is also optional, allowing both to be present or both to be absent simultaneously, when only four of those combinations should even compile. Every place that consumes the type then has to defensively check combinations of optionality that can never actually occur together, which is both extra code and a place bugs hide. A discriminated union makes the compiler enforce that only valid combinations exist, eliminating that class of defensive code entirely.

## Bad

```typescript
interface RequestState<T> {
  data?: T;
  error?: Error;
  isLoading?: boolean;
}

function render(state: RequestState<User>) {
  // Is (data present, error present, isLoading true) valid? Who knows.
  // Every consumer has to guess and defensively check combinations
  // that the type system doesn't actually rule out.
  if (state.isLoading) return "Loading...";
  if (state.error) return `Error: ${state.error.message}`;
  if (state.data) return state.data.name;
  return "Unknown state"; // reachable — and it shouldn't be
}
```

## Good

```typescript
type RequestState<T> =
  | { status: "loading" }
  | { status: "error"; error: Error }
  | { status: "success"; data: T };

function render(state: RequestState<User>): string {
  switch (state.status) {
    case "loading":
      return "Loading...";
    case "error":
      return `Error: ${state.error.message}`;
    case "success":
      return state.data.name;
    // No `default` needed — TypeScript proves the switch is exhaustive,
    // and there is no "unknown state" to fall through to.
  }
}
```

## A Quick Test for "Should This Be Optional?"

Ask: is there a property that's only meaningful *because* another property has a specific value? If so, that's a signal the type should be a discriminated union keyed on that other property, not two independently-optional fields.

| Symptom | Fix |
|---|---|
| Two fields where exactly one is ever set, never both | Discriminated union with a `kind`/`status`/`type` tag |
| A boolean flag that changes which other fields are valid | Union branch per flag value |
| Comments like `// only set when X` next to an optional field | Union branch, `X` as the discriminant |

## See Also

- [type-discriminated-union](type-discriminated-union.md) - Modeling variant data with discriminated unions
- [type-exhaustive-switch](type-exhaustive-switch.md) - Using exhaustiveness checks on discriminated union switches
- [err-result-pattern](err-result-pattern.md) - Modeling success/failure as a discriminated union instead of throwing
