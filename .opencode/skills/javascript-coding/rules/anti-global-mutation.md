# anti-global-mutation

> Don't mutate built-in prototypes or global objects

## Why It Matters

Modifying `Array.prototype`, `String.prototype`, or `Object.prototype` affects ALL objects in the runtime, including those from other libraries and future code. This creates unpredictable bugs, breaks third-party dependencies, and makes the codebase impossible to reason about. It's considered one of the cardinal sins of JavaScript.

## Bad

```js
// Never do this — catastrophic side effects
Array.prototype.last = function () {
  return this[this.length - 1];
};

String.prototype.capitalize = function () {
  return this[0].toUpperCase() + this.slice(1);
};

Object.prototype.isEmpty = function () {
  return Object.keys(this).length === 0;
};

// Breaks for...in loops
for (const key in someObj) {
  console.log(key);  // 'isEmpty' appears!
}
```

## Good

```js
// Use standalone utility functions
function last(arr) {
  return arr[arr.length - 1];
}

function capitalize(str) {
  return str[0].toUpperCase() + str.slice(1);
}

function isEmpty(obj) {
  return Object.keys(obj).length === 0;
}

const arr = [1, 2, 3];
last(arr);  // 3

// For method chaining, use wrapper classes or composition
class ExtendedArray {
  #arr;
  constructor(arr) { this.#arr = arr; }

  last() { return this.#arr[this.#arr.length - 1]; }
  toArray() { return this.#arr; }
}
```

## When Exceptions Apply

Polyfills for standardized APIs (e.g., `Array.prototype.flat` before it was native) are acceptable when feature detection confirms the method is missing. Only polyfill official specifications, never custom methods.

## See Also

- [fn-composition-over-inheritance](./fn-composition-over-inheritance.md) - Composition patterns
- [sec-prototype-pollution](./sec-prototype-pollution.md) - Prototype pollution
