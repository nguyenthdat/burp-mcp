# perf-number-over-parseInt

> Use `Number()` or the unary `+` operator over `parseInt()` for numeric coercion of known-format strings

## Why It Matters

`parseInt()` has surprising behavior: it parses until the first non-digit character (`parseInt('10px')` → 10), stops at decimal points, and returns `NaN` for empty strings. It also requires a radix argument. `Number()` and unary `+` perform strict numeric coercion: they return `NaN` for any non-numeric input, making errors explicit.

## Bad

```js
// parseInt silently truncates — masks bad data
const value = parseInt(userInput);       // No radix — ESLint warning
const pixels = parseInt('10px');         // 10 — error hidden
const float = parseInt('3.14');          // 3 — precision lost
const zero = parseInt('');               // NaN — unexpected

// parseInt with radix is correct but still truncates
const num = parseInt(input, 10);
```

## Good

```js
// Number() — strict, explicit
const value = Number(userInput);         // NaN for '10px' — error caught
const float = Number('3.14');            // 3.14 — correct
const zero = Number('');                 // 0 — consistent
const nan = Number('abc');               // NaN

// Unary + — concise
const value = +userInput;

// For integers specifically
const int = Number.parseInt(input, 10);  // Number.parseInt is preferred over global parseInt

// For safe integer parsing with validation
function parseSafeInt(input, fallback = 0) {
  const num = Number(input);
  return Number.isSafeInteger(num) ? num : fallback;
}
```

## When to Use Each

| Function | Use Case |
|----------|----------|
| `Number()` / `+` | Converting strings that should be fully numeric |
| `Number.parseInt()` | Extracting leading integers from strings (rare) |
| `Number.parseFloat()` | Extracting leading floats from strings (rare) |
| `BigInt()` | Large integers beyond 2^53 |

## When Exceptions Apply

Use `parseInt` only when you specifically want to parse leading integer digits from a string that may contain non-numeric suffix (e.g., CSS values like `'10px'`). In most cases, `Number()` is correct.

## See Also

- [type-bigint-precision](./type-bigint-precision.md) - BigInt for large numbers
- [type-avoid-implicit-coercion](./type-avoid-implicit-coercion.md) - Strict equality
