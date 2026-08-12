# imm-prefer-const

> Default to `const`; use `let` only when a binding is reassigned

## Why It Matters

`const` communicates intent to every future reader: this binding is assigned once and never changes. When everything is `let` by default, readers must trace the entire enclosing scope to know whether a value mutates, which slows down review and hides bugs where a variable is reassigned accidentally. `const` also lets the TypeScript compiler narrow types more aggressively (a `const` string literal is typed as the literal, not `string`), and it catches accidental reassignment as a compile error instead of a silent runtime surprise.

## Bad

```typescript
let apiUrl = "https://api.example.com/v1";
let maxRetries = 3;

function buildRequest(path: string) {
  let url = apiUrl + path;
  let headers = { "Content-Type": "application/json" };
  return { url, headers };
}

// Nothing here is ever reassigned, but every reader has to check.
```

## Good

```typescript
const apiUrl = "https://api.example.com/v1";
const maxRetries = 3;

function buildRequest(path: string) {
  const url = apiUrl + path;
  const headers = { "Content-Type": "application/json" };
  return { url, headers };
}

// Reserve `let` for genuine reassignment.
let attempts = 0;
while (attempts < maxRetries) {
  attempts += 1;
}
```

## Enforcing It With Lint

```jsonc
// .eslintrc.json (or eslint.config.js "rules")
{
  "rules": {
    "prefer-const": ["error", { "destructuring": "all" }],
    "no-var": "error"
  }
}
```

`"destructuring": "all"` avoids false positives: a destructuring assignment is only flagged if *every* destructured variable is never reassigned, since `const { a, b } = obj` can't selectively apply `const` to just one of `a`/`b`.

## When let Is Still Correct

- Loop counters and accumulators (`for (let i = 0; ...)`, running totals).
- A value computed conditionally across multiple branches before first use, when restructuring into a single expression would hurt readability.
- State that a closure captures and updates over time (e.g., a debounce timer handle).

## See Also

- [anti-var-usage](anti-var-usage.md) - Never use `var`; it has function scope and no temporal dead zone protection
- [imm-spread-not-mutate](imm-spread-not-mutate.md) - Create updated copies with spread/rest instead of mutating in place
- [type-const-assertion](type-const-assertion.md) - Use `as const` to freeze literal values at the type level
