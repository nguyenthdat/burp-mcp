# type-function-overloads

> Use overload signatures to model functions with varying call shapes

## Why It Matters

A single signature with unions and optional parameters (`function parse(input: string | Buffer, opts?: Options): string | Buffer`) forces every caller to deal with a return type that includes possibilities that can't actually happen for the arguments they passed. Overload signatures let you declare multiple precise call shapes — mapping specific argument combinations to specific, narrower return types — so callers get exactly the type their particular call produces, without a spurious union or manual narrowing afterward.

## Bad

```typescript
function createElement(tag: string): HTMLElement | HTMLCanvasElement | HTMLInputElement {
  return document.createElement(tag);
}

const canvas = createElement("canvas");
canvas.getContext("2d"); // Error: getContext doesn't exist on HTMLElement | HTMLInputElement | HTMLCanvasElement
// Caller has to narrow manually even though "canvas" unambiguously determines the return type
```

## Good

```typescript
function createElement(tag: "canvas"): HTMLCanvasElement;
function createElement(tag: "input"): HTMLInputElement;
function createElement(tag: string): HTMLElement;
function createElement(tag: string): HTMLElement {
  return document.createElement(tag);
}

const canvas = createElement("canvas"); // typed as HTMLCanvasElement
canvas.getContext("2d"); // OK, no narrowing needed

const input = createElement("input"); // typed as HTMLInputElement
input.value = "hello"; // OK

const generic = createElement("div"); // typed as HTMLElement (falls through to the general overload)
```

## Overloads for Optional/Dependent Parameters

Overloads also help when a later parameter's type depends on an earlier one, which a single union signature can't express precisely:

```typescript
function query(sql: string, params: number[]): Promise<Row[]>;
function query(sql: string, params: number[], options: { stream: true }): AsyncIterable<Row>;
function query(
  sql: string,
  params: number[],
  options?: { stream: true },
): Promise<Row[]> | AsyncIterable<Row> {
  return options?.stream ? streamRows(sql, params) : collectRows(sql, params);
}
```

## Ordering Rules

TypeScript picks the *first* overload that matches the call, so overloads must be ordered from most specific to most general — the implementation signature (the last one, with the body) is not itself part of the public overload list and must be broad enough to cover every declared overload.

## When to Prefer a Union Signature Instead

| Situation | Prefer |
|---|---|
| Return type genuinely depends on argument shape/value | Overloads |
| Return type is the same regardless of which union member is passed | Single signature with a union parameter |
| Generic type parameters can express the relationship | Generics (often clearer than overloads) |

## See Also

- [api-function-overload-order](api-function-overload-order.md) - Ordering overload signatures from most to least specific
- [api-explicit-return-types](api-explicit-return-types.md) - Always declare explicit return types on exported functions
- [type-generic-constraints](type-generic-constraints.md) - Constrain generic type parameters with extends instead of leaving them unbounded
