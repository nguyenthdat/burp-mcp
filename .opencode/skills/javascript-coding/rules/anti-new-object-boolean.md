# anti-new-object-boolean

> Don't use `new Boolean()`, `new String()`, or `new Number()` — use primitives or factory functions

## Why It Matters

`new Boolean(false)` creates a `Boolean` object, which is truthy (all objects are truthy), making `if (new Boolean(false))` evaluate to `true` — the exact opposite of what you'd expect. The primitive wrapper constructors create objects instead of primitives, which behave unexpectedly in comparisons and type checks. Use literal syntax or factory functions.

## Bad

```js
// new Boolean — truthy trap
const isActive = new Boolean(false);
if (isActive) {
  console.log('Active');  // This runs! new Boolean(false) is truthy
}

// new String — typeof object, not string
const name = new String('Alice');
typeof name;  // 'object' — not 'string'!
name === 'Alice';  // false — object !== string

// new Number — same issue
const count = new Number(42);
typeof count;  // 'object'
count === 42;  // false
```

## Good

```js
// Primitives — behave as expected
const isActive = false;
if (isActive) { /* Won't run */ }

const name = 'Alice';
typeof name;  // 'string'
name === 'Alice';  // true

const count = 42;
typeof count;  // 'number'

// Use factory functions if you need objects
const strObj = String('hello');   // Returns primitive
const numObj = Number('42');      // Returns primitive
const boolObj = Boolean(1);       // Returns primitive
```

## When Exceptions Apply

There are no exceptions. The primitive wrapper constructors (`new Boolean`, `new String`, `new Number`) should never be used. ESLint's `no-new-wrappers` rule enforces this.

## See Also

- [type-avoid-implicit-coercion](./type-avoid-implicit-coercion.md) - Strict equality
- [perf-number-over-parseInt](./perf-number-over-parseInt.md) - Number() over parseInt()
