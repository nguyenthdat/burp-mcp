# anti-overly-smart-code

> Don't sacrifice readability for cleverness — write code for humans, not machines

## Why It Matters

Clever code impresses no one. It confuses teammates, hides bugs, and makes refactoring dangerous. The best code is boring — it does exactly what it looks like it does, with no surprises. A 3-line solution that everyone understands is better than a 1-line puzzle that requires 5 minutes of deciphering.

## Bad

```js
// Clever but incomprehensible
const result = ~~(+(+!![] + [] + +!![]) + (+!![] + []));

// Overly compressed logic
const fn = (a, b) => (a ^ b) && (a = (b += a -= b) - a) || a;

// Unnecessary code golf
const isEven = n => !(n & 1);  // Works but `n % 2 === 0` is clearer
```

## Good

```js
// Clear, boring, understandable
function isEven(n) {
  return n % 2 === 0;
}

function swap(a, b) {
  const temp = a;
  a = b;
  b = temp;
  return [a, b];
}

// Use built-in methods instead of bit tricks
Math.floor(3.7);  // Not ~~3.7
parseInt('10', 10);  // Not +'10'
```

## When Exceptions Apply

Bitwise operations are appropriate in performance-critical numeric code (graphics, cryptography, binary protocols) — but add a comment explaining what's happening.

## See Also

- [anti-nested-ternary](./anti-nested-ternary.md) - Don't nest ternaries
- [doc-inline-comments-why](./doc-inline-comments-why.md) - Comment why, not what
