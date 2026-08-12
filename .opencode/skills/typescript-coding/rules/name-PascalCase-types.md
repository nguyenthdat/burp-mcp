# name-PascalCase-types

> Use `PascalCase` for types, interfaces, classes, and enums

## Why It Matters

Casing is the fastest signal TypeScript code gives about what kind of thing an identifier is. When types, interfaces, classes, and enums are consistently `PascalCase` and values are consistently `camelCase`, a reader can tell `User` is a type and `user` is a value without looking anything up — which matters constantly, since the same word often names both (`interface Order` next to `const order: Order = ...`). Breaking this convention (lowercase interfaces, snake_case enum members) removes that signal and makes type-vs-value confusion more likely, especially in files with heavy generic or structural typing.

## Bad

```typescript
interface user {
  id: string;
  displayName: string;
}

type api_response<T> = {
  data: T;
  status: number;
};

class httpClient {
  constructor(private baseUrl: string) {}
}

enum order_status {
  pending,
  shipped,
  delivered,
}
```

## Good

```typescript
interface User {
  id: string;
  displayName: string;
}

type ApiResponse<T> = {
  data: T;
  status: number;
};

class HttpClient {
  constructor(private baseUrl: string) {}
}

enum OrderStatus {
  Pending,
  Shipped,
  Delivered,
}
```

## Applies To Generic Type Aliases And Enum Members Too

```typescript
type Result<T, E = Error> = { ok: true; value: T } | { ok: false; error: E };

enum LogLevel {
  Debug,
  Info,
  Warning,
  Error,
}

// Not: type result<t, e> = ...
// Not: enum logLevel { debug, info, ... }
```

## Enforcing With ESLint

```jsonc
{
  "rules": {
    "@typescript-eslint/naming-convention": [
      "error",
      { "selector": "typeLike", "format": ["PascalCase"] },
      { "selector": "enumMember", "format": ["PascalCase"] }
    ]
  }
}
```

## See Also

- [name-camelCase-vars](name-camelCase-vars.md) - Use `camelCase` for variables and functions
- [name-interface-no-I-prefix](name-interface-no-I-prefix.md) - Don't prefix interfaces with `I`
- [api-avoid-enum-const-object](api-avoid-enum-const-object.md) - Prefer a `const` object plus derived union over `enum` in most cases
