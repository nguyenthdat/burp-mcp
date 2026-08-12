# name-camelCase-vars

> Use `camelCase` for variables and functions

## Why It Matters

Consistent casing is a low-cost, high-value convention: it lets a reader distinguish a variable/function from a type/class/enum (`PascalCase`) or a true constant (`SCREAMING_SNAKE_CASE`) at a glance, without needing to know what the identifier refers to first. Mixed casing conventions within the same codebase (`snake_case` next to `camelCase` next to inconsistent capitalization) force readers to context-switch mid-line and make it harder to spot typos, since there's no single expected shape to compare against.

## Bad

```typescript
const user_name = "ada";
const Total_Price = 42.5;
let IsActive = true;

function Calculate_Tax(amount: number) {
  return amount * 0.08;
}

function get_user_by_id(UserId: string) {
  // ...
}
```

## Good

```typescript
const userName = "ada";
const totalPrice = 42.5;
let isActive = true;

function calculateTax(amount: number) {
  return amount * 0.08;
}

function getUserById(userId: string) {
  // ...
}
```

## Casing Convention Summary

| Identifier kind | Convention | Example |
|---|---|---|
| Variable, function, method | `camelCase` | `userName`, `calculateTax()` |
| Type, interface, class, enum | `PascalCase` | `UserProfile`, `HttpClient` |
| True module-level constant | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES` |
| Private class field | `camelCase` with `#`/`private` | `#requestCount` |

## Enforcing With `@typescript-eslint/naming-convention`

```jsonc
{
  "rules": {
    "@typescript-eslint/naming-convention": [
      "error",
      { "selector": "variableLike", "format": ["camelCase"] },
      { "selector": "typeLike", "format": ["PascalCase"] },
      {
        "selector": "variable",
        "modifiers": ["const", "global"],
        "types": ["boolean", "string", "number"],
        "format": ["camelCase", "UPPER_CASE"]
      }
    ]
  }
}
```

This lets both regular `camelCase` locals and true `SCREAMING_SNAKE_CASE` module constants pass, while still catching stray `snake_case` or `PascalCase` variables.

## See Also

- [name-PascalCase-types](name-PascalCase-types.md) - Use `PascalCase` for types, interfaces, classes, and enums
- [name-SCREAMING-const](name-SCREAMING-const.md) - Use `SCREAMING_SNAKE_CASE` for true module-level constants
- [lint-typescript-eslint-recommended](lint-typescript-eslint-recommended.md) - Adopt the typescript-eslint recommended rule set as a baseline
