# type-template-literal

> Use template literal types to constrain string patterns

## Why It Matters

A bare `string` type accepts anything, so a function expecting a CSS unit, an event name, or a route path has no compiler protection against typos like `"1o0px"` or `"onClcik"`. Template literal types let you express string *patterns* — prefixes, suffixes, and interpolated unions — as part of the type system, turning a class of stringly-typed bugs into autocomplete hints and compile errors instead of runtime failures.

## Bad

```typescript
function setWidth(value: string) {
  element.style.width = value;
}

setWidth("100"); // compiles, but CSS silently ignores a unit-less width
setWidth("100xp"); // typo, compiles, silently wrong

type EventName = string;
function on(event: EventName, handler: () => void) {
  /* ... */
}
on("onClcik", () => {}); // typo compiles fine
```

## Good

```typescript
type CssUnit = "px" | "em" | "rem" | "%";
type CssLength = `${number}${CssUnit}`;

function setWidth(value: CssLength) {
  element.style.width = value;
}

setWidth("100px"); // OK
setWidth("100xp"); // Error: not assignable to type 'CssLength'

type EventBase = "click" | "focus" | "blur";
type EventName = `on${Capitalize<EventBase>}`; // "onClick" | "onFocus" | "onBlur"

function on(event: EventName, handler: () => void) {
  /* ... */
}
on("onClick", () => {}); // OK, autocompletes
on("onClcik", () => {}); // Error: not assignable
```

## Deriving Types From Route or Key Tables

```typescript
const routes = {
  home: "/",
  userProfile: "/users/:id",
  orgSettings: "/orgs/:orgId/settings",
} as const;

type RouteKey = keyof typeof routes;

// Extract path params from a template literal route string
type ExtractParams<Path extends string> =
  Path extends `${string}:${infer Param}/${infer Rest}`
    ? { [K in Param | keyof ExtractParams<`/${Rest}`>]: string }
    : Path extends `${string}:${infer Param}`
      ? { [K in Param]: string }
      : Record<string, never>;

type OrgSettingsParams = ExtractParams<(typeof routes)["orgSettings"]>;
// { orgId: string }
```

## Built-In String Manipulation Types

| Utility | Effect |
|---|---|
| `Uppercase<S>` | `"abc"` -> `"ABC"` |
| `Lowercase<S>` | `"ABC"` -> `"abc"` |
| `Capitalize<S>` | `"click"` -> `"Click"` |
| `Uncapitalize<S>` | `"Click"` -> `"click"` |

These compose naturally with template literals, as shown in the `EventName` example above.

## See Also

- [type-utility-types](type-utility-types.md) - Prefer built-in utility types over hand-rolled equivalents
- [anti-stringly-typed-data](anti-stringly-typed-data.md) - Avoid representing structured data as loosely-typed strings
- [type-branded-nominal](type-branded-nominal.md) - Use branded/nominal types to distinguish primitives with the same runtime type
