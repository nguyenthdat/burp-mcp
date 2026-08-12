# anti-arguments-object

> Don't use the `arguments` object — use rest parameters (`...args`) instead

## Why It Matters

The `arguments` object is array-like but not an array (no `.map`, `.filter`, `.forEach`), doesn't work in arrow functions, and obscures the function's parameter signature. Rest parameters produce a real array, work in all function types, and make the variadic nature of the function explicit in its signature.

## Bad

```js
// arguments — not a real array, no arrow functions
function sum() {
  // Must convert to array manually
  const args = Array.prototype.slice.call(arguments);
  return args.reduce((total, n) => total + n, 0);
}

// Arrow functions don't have arguments
const fn = () => {
  console.log(arguments);  // ReferenceError or parent scope's arguments
};
```

## Good

```js
// Rest parameters — real array, works everywhere
function sum(...numbers) {
  return numbers.reduce((total, n) => total + n, 0);
}

// Arrow functions work fine
const sum = (...numbers) => numbers.reduce((total, n) => total + n, 0);

// Clear signature with named params + rest
function format(first, last, ...middle) {
  return `${first} ${middle.join(' ')} ${last}`;
}
```

## When Exceptions Apply

There are no exceptions for new code. The `arguments` object is a legacy API. ESLint's `prefer-rest-params` rule enforces this.

## See Also

- [fn-rest-spread](./fn-rest-spread.md) - Rest/spread operators
- [lint-no-var](./lint-no-var.md) - No var (arguments is similarly legacy)
