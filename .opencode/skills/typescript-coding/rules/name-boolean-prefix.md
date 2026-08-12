# name-boolean-prefix

> Prefix booleans with `is`/`has`/`can`/`should`

## Why It Matters

A boolean named without a predicate prefix (`active`, `admin`, `visible`) reads ambiguously at the call site — `if (active)` could just as easily be a status enum, a count, or an object. Prefixing with `is`/`has`/`can`/`should` (`isActive`, `hasAdmin`, `canEdit`, `shouldRetry`) makes the variable read as a yes/no question in every usage, which both documents the type without needing to check it and makes conditionals read like natural language.

## Bad

```typescript
interface User {
  active: boolean;
  admin: boolean;
  subscription: boolean;
}

function process(user: User) {
  if (user.active && !user.admin) {
    // ...
  }
}

let retry = true;
let valid = checkForm(data);
let visible = false;
```

## Good

```typescript
interface User {
  isActive: boolean;
  isAdmin: boolean;
  hasSubscription: boolean;
}

function process(user: User) {
  if (user.isActive && !user.isAdmin) {
    // ...
  }
}

let shouldRetry = true;
let isValid = checkForm(data);
let isVisible = false;
```

## Prefix-to-Meaning Guide

| Prefix | Signals |
|---|---|
| `is` | A state or classification (`isLoading`, `isEmpty`, `isValid`) |
| `has` | Possession of something (`hasPermission`, `hasChildren`, `hasError`) |
| `can` | A capability or permission (`canEdit`, `canDelete`, `canRetry`) |
| `should` | A directive/decision the caller should act on (`shouldRefresh`, `shouldRender`) |

## Function Names That Return Booleans Follow The Same Rule

```typescript
function isValidEmail(email: string): boolean { /* ... */ }
function hasPermission(user: User, action: string): boolean { /* ... */ }
function canTransition(from: State, to: State): boolean { /* ... */ }

// Not: function checkEmail(...), function permission(...), function transition(...)
// These read ambiguously about whether they return a boolean at all.
```

## Enforcing With ESLint

`eslint-plugin-unicorn`'s `no-boolean-literal-compare` combined with typescript-eslint's `naming-convention` selector for boolean-typed variables can flag boolean identifiers that lack a recognized prefix:

```jsonc
{
  "rules": {
    "@typescript-eslint/naming-convention": [
      "error",
      {
        "selector": "variable",
        "types": ["boolean"],
        "format": ["PascalCase"],
        "prefix": ["is", "has", "can", "should", "did", "will"]
      }
    ]
  }
}
```

## See Also

- [name-verb-noun-functions](name-verb-noun-functions.md) - Name functions with a leading verb describing the action they perform
- [name-avoid-abbreviations](name-avoid-abbreviations.md) - Avoid unclear abbreviations in identifiers
- [type-discriminated-union](type-discriminated-union.md) - Model mutually-exclusive states with a discriminated union instead of multiple booleans
