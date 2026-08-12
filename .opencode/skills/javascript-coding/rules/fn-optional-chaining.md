# fn-optional-chaining

> Use optional chaining (`?.`) and nullish coalescing (`??`) for safe property access

## Why It Matters

Accessing nested properties on potentially null/undefined values causes `TypeError: Cannot read properties of undefined`. Optional chaining (`?.`) short-circuits to `undefined` safely. Combined with nullish coalescing (`??`), it provides clean default values without the pitfalls of `||` (which treats `0`, `''`, `false` as falsy).

## Bad

```js
// Verbose and error-prone chained checks
const city = user && user.address && user.address.city;

// || treats falsy values incorrectly
const count = user.count || 0;       // 0 is overwritten
const name = user.name || 'Guest';    // '' is overwritten
const enabled = settings.enabled || true;  // false is overwritten
```

## Good

```js
// Optional chaining — clean and safe
const city = user?.address?.city;

// Nullish coalescing — only catches null/undefined
const count = user.count ?? 0;
const name = user.name ?? 'Guest';
const enabled = settings.enabled ?? true;

// Combined usage
const cityName = user?.address?.city ?? 'Unknown';
```

## Advanced Patterns

```js
// Optional method calls
const result = obj.method?.();

// Optional array access
const first = arr?.[0];

// Optional dynamic property
const value = obj?.[dynamicKey];

// Chained with function calls
const formatted = user?.getName?.()?.toUpperCase() ?? 'ANONYMOUS';
```

## Deeply Nested Access with Destructuring

```js
// Combine optional chaining with destructuring
const { city, country } = user?.address ?? {};
const location = city ? `${city}, ${country}` : 'Unknown';
```

## When Exceptions Apply

Don't use optional chaining when `null`/`undefined` would indicate a programming error that should throw. Use it for data that's legitimately optional (API responses, user input, configuration).

## See Also

- [fn-nullish-coalescing](./fn-nullish-coalescing.md) - ?? over ||
- [fn-destructure](./fn-destructure.md) - Destructuring patterns
