# type-exhaustive-switch

> Enforce exhaustiveness checks with a `never` assertion

## Why It Matters

When you add a new variant to a union or enum, every `switch` statement that branches on it should be updated to handle the new case. Without an exhaustiveness check, a missing branch silently falls through (or hits a `default` that papers over the gap), and the bug isn't discovered until it runs in production. Assigning the unmatched value to a variable typed `never` in the default case makes the compiler flag any switch that forgot a variant, the moment the union grows.

## Bad

```typescript
type Shape =
  | { kind: "circle"; radius: number }
  | { kind: "square"; side: number };

function area(shape: Shape): number {
  switch (shape.kind) {
    case "circle":
      return Math.PI * shape.radius ** 2;
    case "square":
      return shape.side ** 2;
    // No default, no error — but also no protection if a variant is added later
  }
  return 0; // silently wrong for any shape that reaches here
}

// Later, someone adds a variant:
type Shape2 =
  | { kind: "circle"; radius: number }
  | { kind: "square"; side: number }
  | { kind: "triangle"; base: number; height: number };
// `area` above still compiles and silently returns 0 for triangles
```

## Good

```typescript
type Shape =
  | { kind: "circle"; radius: number }
  | { kind: "square"; side: number }
  | { kind: "triangle"; base: number; height: number };

function assertUnreachable(value: never): never {
  throw new Error(`unhandled case: ${JSON.stringify(value)}`);
}

function area(shape: Shape): number {
  switch (shape.kind) {
    case "circle":
      return Math.PI * shape.radius ** 2;
    case "square":
      return shape.side ** 2;
    case "triangle":
      return 0.5 * shape.base * shape.height;
    default:
      // Compile error the moment a Shape variant isn't handled above:
      // "Argument of type '{ kind: "triangle"; ... }' is not assignable to parameter of type 'never'"
      return assertUnreachable(shape);
  }
}
```

## Exhaustiveness Outside `switch`

The same `never` trick works for `if`/`else if` chains and object lookup maps:

```typescript
function areaViaMap(shape: Shape): number {
  const handlers: Record<Shape["kind"], (s: Shape) => number> = {
    circle: (s) => Math.PI * (s as Extract<Shape, { kind: "circle" }>).radius ** 2,
    square: (s) => (s as Extract<Shape, { kind: "square" }>).side ** 2,
    triangle: (s) => {
      const t = s as Extract<Shape, { kind: "triangle" }>;
      return 0.5 * t.base * t.height;
    },
  };
  return handlers[shape.kind](shape);
}
// If a variant is missing from `handlers`, Record<Shape["kind"], ...> fails to compile.
```

## See Also

- [type-discriminated-union](type-discriminated-union.md) - Model variants with discriminated unions and a common tag field
- [type-narrow-guards](type-narrow-guards.md) - Use user-defined type guards to narrow union types safely
- [anti-deeply-nested-ternary](anti-deeply-nested-ternary.md) - Avoid deeply nested ternary expressions
