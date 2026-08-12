# type-bigint-precision

> Use `BigInt` for integers beyond `Number.MAX_SAFE_INTEGER` (2^53 - 1)

## Why It Matters

JavaScript's `Number` type is a 64-bit floating point (IEEE 754) with 53 bits of integer precision. Integers above 2^53 lose precision silently: `9007199254740993 === 9007199254740992` is `true`. `BigInt` provides arbitrary-precision integers for cryptography, financial calculations, large IDs, and timestamp arithmetic.

## Bad

```js
// Number loses precision silently
const largeId = 9007199254740993;  // Actually stored as 9007199254740992
console.log(largeId);  // 9007199254740992 — wrong!

const maxInt = Number.MAX_SAFE_INTEGER;  // 9007199254740991
maxInt + 1 === maxInt + 2;  // true — both round to the same value

// Financial calculations with floats
const balance = 0.1 + 0.2;  // 0.30000000000000004
```

## Good

```js
// BigInt — exact integer representation
const largeId = 9007199254740993n;  // Exact
console.log(largeId);  // 9007199254740993n — correct!

const result = largeId + 1n;
console.log(result);  // 9007199254740994n

// Financial calculations with BigInt (cents)
const price = 1999n;  // $19.99 in cents
const quantity = 3n;
const total = price * quantity;  // 5997n — exact
```

## Mixing BigInt and Number

```js
// Cannot mix BigInt and Number in arithmetic
// const x = 1n + 1;  // TypeError!

// Explicit conversion
const x = 1n + BigInt(1);       // 2n
const y = Number(1n) + 1;       // 2

// Number() on BigInt > MAX_SAFE_INTEGER loses precision
const big = 9007199254740993n;
Number(big);  // 9007199254740992 — truncated!

// Comparison works
1n === 1;  // false (different types)
1n == 1;   // true (loose equality)
```

## JSON Serialization

```js
// BigInt is not JSON-serializable by default
// JSON.stringify({ id: 123n });  // TypeError!

// Use a replacer
function bigintReplacer(key, value) {
  return typeof value === 'bigint' ? value.toString() : value;
}
JSON.stringify({ id: 123n }, bigintReplacer);  // '{"id":"123"}'

// Or use a custom toJSON
BigInt.prototype.toJSON = function () {
  return this.toString();
};
JSON.stringify({ id: 123n });  // '{"id":"123"}'
```

## When Exceptions Apply

For most web development and API work, `Number` is sufficient. Use `BigInt` for cryptography, high-precision financial math, unique IDs from distributed systems (Snowflake IDs), and when interfacing with systems that use 64-bit integers.

## See Also

- [type-avoid-implicit-coercion](./type-avoid-implicit-coercion.md) - Strict equality
- [fn-temporal-over-date](./fn-temporal-over-date.md) - Temporal API for dates
