# fn-nullish-coalescing

> Use `??` instead of `||` when only `null`/`undefined` should trigger the default

## Why It Matters

`||` treats every JavaScript falsy value — `0`, `""`, `false`, `NaN`, `null`, `undefined` — as "absent" and replaces it with the fallback. That's almost never what you actually mean when the intent is "use this default only if the value wasn't provided." `??` only falls through on `null` or `undefined`, so legitimate falsy values like `0` quantities, empty strings, or an explicit `false` flag are preserved instead of being silently overwritten — a distinction that has caused real production bugs (a `0` price becoming a default price, an unchecked `false` becoming `true`).

## Bad

```typescript
function getPageSize(config: { pageSize?: number }): number {
  return config.pageSize || 20; // if pageSize is explicitly 0, you get 20, not 0
}

function getDiscount(order: { discountPercent?: number }): number {
  return order.discountPercent || 0.1; // an intentional 0% discount becomes 10%
}

function isFeatureEnabled(flags: { darkMode?: boolean }): boolean {
  return flags.darkMode || true; // an explicit `false` is overridden to `true` — bug
}
```

## Good

```typescript
function getPageSize(config: { pageSize?: number }): number {
  return config.pageSize ?? 20; // only replaces null/undefined; 0 is respected
}

function getDiscount(order: { discountPercent?: number }): number {
  return order.discountPercent ?? 0.1; // an explicit 0 is respected
}

function isFeatureEnabled(flags: { darkMode?: boolean }): boolean {
  return flags.darkMode ?? true; // an explicit false is respected
}
```

## `??` vs `||` — When Each Is Correct

| Expression | Falls back on | Use when |
|---|---|---|
| `a ?? b` | `null`, `undefined` only | `a` is a value where `0`, `""`, or `false` are meaningful, valid data |
| `a \|\| b` | any falsy value | You genuinely want to replace every falsy value, e.g. `input.trim() \|\| "N/A"` for a "blank means missing" string |

## Nullish Assignment Operator

The `??=` operator pairs naturally with `??` for "assign a default only if currently nullish":

```typescript
interface Options {
  timeout?: number;
}

function withDefaults(opts: Options): Required<Options> {
  opts.timeout ??= 5000; // only assigns if opts.timeout is null/undefined
  return opts as Required<Options>;
}
```

## Lint Enforcement

```jsonc
{
  "rules": {
    "@typescript-eslint/prefer-nullish-coalescing": "error"
  }
}
```

This rule flags `||` used in a default-value position where the left side's type includes a legitimate falsy value other than `null`/`undefined`, which is exactly the bug pattern shown above.

## See Also

- [fn-optional-chaining](fn-optional-chaining.md) - Use optional chaining (`?.`) instead of manual nested null checks
- [type-strict-null-checks](type-strict-null-checks.md) - Enable `strictNullChecks` so `null`/`undefined` are tracked in the type system
- [anti-loose-equality](anti-loose-equality.md) - Use `===`/`!==` instead of `==`/`!=` to avoid type-coercion surprises
