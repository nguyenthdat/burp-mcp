# anti-parseint-no-radix

> Always pass the radix (base) argument to `parseInt()`

## Why It Matters

`parseInt()` without a radix guesses the base from the string: strings starting with `0x` are hexadecimal, `0` was octal in older engines. This produces unpredictable results with user input. Always explicitly pass `10` as the radix, or better yet, use `Number()` which always assumes base 10.

## Bad

```js
// No radix — behavior depends on input
const value = parseInt(userInput);      // "0x10" → 16 (hex!), "010" → 10 (or 8 in old engines)

// parseInt on non-string inputs
const num = parseInt(0.000005);  // '0.000005' → 0 (parsed as "0.000005" → stops at ".")
```

## Good

```js
// Always pass radix
const value = parseInt(userInput, 10);

// Or use Number() for strict base-10 conversion
const value = Number(userInput);

// Number.parseInt is clearer about intent
const value = Number.parseInt(userInput, 10);

// For robust parsing with validation
function parseSafeInt(input, fallback = 0) {
  const num = parseInt(input, 10);
  return Number.isNaN(num) ? fallback : num;
}
```

## ESLint Enforcement

```js
// eslint.config.mjs
{
  rules: {
    'radix': ['error', 'always'],
  },
}
```

## When Exceptions Apply

When you genuinely need to parse a non-decimal string: `parseInt('ff', 16)` → 255. Always specify the radix, even for non-decimal.

## See Also

- [perf-number-over-parseInt](./perf-number-over-parseInt.md) - Number() over parseInt()
- [type-bigint-precision](./type-bigint-precision.md) - BigInt for large numbers
