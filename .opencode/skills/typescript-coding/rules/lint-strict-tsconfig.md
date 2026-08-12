# lint-strict-tsconfig

> Enable `strict: true` and other strictness flags in `tsconfig.json`

## Why It Matters

TypeScript's checks are opt-in and additive; without `strict: true`, the compiler allows implicit `any` parameters, treats `null`/`undefined` as assignable to every type, and skips several other checks that catch entire classes of runtime bugs before they ship. `strict` is a bundle of individually-toggleable flags (`strictNullChecks`, `noImplicitAny`, `strictFunctionTypes`, `strictBindCallApply`, `strictPropertyInitialization`, and more) that each closes a specific hole; enabling it is the single highest-leverage TypeScript configuration decision in a project, and it's dramatically cheaper to turn on at project start than to retrofit onto a large non-strict codebase later.

## Bad

```jsonc
// tsconfig.json — strict mode off (or simply not set)
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext"
  }
}
```

```typescript
function greet(name) { // implicit any parameter, no error
  return 'Hello, ' + name.toUpperCase();
}

let value: string;
value = null; // allowed without strictNullChecks — crashes elsewhere at runtime
```

## Good

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "noImplicitOverride": true,
    "exactOptionalPropertyTypes": true
  }
}
```

```typescript
function greet(name: string) { // now required — compiler catches missing types
  return 'Hello, ' + name.toUpperCase();
}

let value: string;
value = null; // Error: Type 'null' is not assignable to type 'string'.
```

## What `strict: true` Actually Enables

| Flag | Catches |
|---|---|
| `strictNullChecks` | Using `null`/`undefined` where not explicitly allowed |
| `noImplicitAny` | Parameters/variables inferred as `any` due to missing annotations |
| `strictFunctionTypes` | Unsound function parameter variance |
| `strictBindCallApply` | Incorrect arguments to `.bind`/`.call`/`.apply` |
| `strictPropertyInitialization` | Class fields declared but never assigned in the constructor |
| `alwaysStrict` | Emits `"use strict"`, disallows sloppy-mode JS semantics |
| `useUnknownInCatchVariables` | `catch (e)` is typed `unknown`, not `any` |

Beyond `strict`, also consider `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes` — both are excluded from `strict` for backward-compatibility reasons but catch real bugs in most codebases.

## See Also

- [lint-no-unchecked-indexed-access](lint-no-unchecked-indexed-access.md) - Enable `noUncheckedIndexedAccess` in `tsconfig.json`
- [type-strict-null-checks](type-strict-null-checks.md) - Rely on `strictNullChecks` instead of manual null checks everywhere
- [err-typed-catch-unknown](err-typed-catch-unknown.md) - Treat caught errors as `unknown`, not `any`
