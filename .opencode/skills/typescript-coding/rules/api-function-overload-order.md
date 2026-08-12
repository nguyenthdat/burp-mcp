# api-function-overload-order

> Order overload signatures from most specific to most general

## Why It Matters

TypeScript resolves a call against function overload signatures in the order they're declared, picking the *first* one the call arguments are assignable to — not the "best" or most specific match. If a general overload (accepting `unknown`, a wide union, or optional parameters) is declared before a more specific one, calls that should resolve to the specific, precisely-typed overload will instead match the general one first, silently giving callers a less precise (or outright wrong) return type.

## Bad

```typescript
function getElement(selector: string): Element | null;
function getElement(selector: string, root: ParentNode): Element | null;
// General, string-only-but-different-purpose overload declared last
function getElement(id: number): Element | null; // never reachable as intended!

// Call:
getElement(42);
// TypeScript tries overload 1 first: does `number` match `string`? No.
// Tries overload 2: does `number` match `(string, ParentNode)`? No, wrong arity.
// Falls through to overload 3 — this one happens to work here, but in
// less contrived examples with overlapping parameter types, ordering
// bugs like this cause the wrong overload (and wrong return type) to
// be selected silently.
```

## Good

```typescript
// Most specific first: number id lookup
function getElement(id: number): Element | null;
// Then string selector with optional root
function getElement(selector: string, root?: ParentNode): Element | null;
function getElement(target: number | string, root?: ParentNode): Element | null {
  if (typeof target === "number") {
    return document.getElementById(String(target));
  }
  return (root ?? document).querySelector(target);
}
```

## A Concrete Ordering Rule of Thumb

1. Overloads with more specific literal/union parameter types before overloads with wider types (`string` before `string | number`, a literal union before `string`).
2. Overloads with more required parameters before overloads with fewer/optional parameters, when both could otherwise match the same call.
3. The general, catch-all implementation signature (the one with the actual function body) is never itself part of the public overload set — TypeScript hides it from callers and only the declared overload signatures above it are visible.

## Enforcing With ESLint

```jsonc
{
  "rules": {
    "@typescript-eslint/adjacent-overload-signatures": "error"
  }
}
```

This doesn't check *ordering by specificity* (that's a design judgment), but it does require overloads to be grouped together rather than scattered through a file, which makes ordering mistakes far easier to spot in review.

## See Also

- [api-generic-defaults](api-generic-defaults.md) - Give generic type parameters sensible defaults where one exists
- [type-narrow-guards](type-narrow-guards.md) - Writing type guards that narrow union parameters correctly
- [api-accept-narrow-return-wide](api-accept-narrow-return-wide.md) - Accept the most general input types callers already have; return the most specific types
