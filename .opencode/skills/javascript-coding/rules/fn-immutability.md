# fn-immutability

> Prefer const, spread operator, and Object.freeze() over mutation

## Why It Matters

Mutable state is the root of many bugs: unexpected side effects, race conditions, and hard-to-trace state changes. Immutable data patterns make code predictable, easier to debug, and safer in concurrent contexts. `const` prevents reassignment, spread creates new objects/arrays, and `Object.freeze()` enforces immutability.

## Bad

```js
// Mutation — original data is modified
function addToCart(cart, item) {
  cart.push(item);  // Mutates the original array
  return cart;
}

function updateUser(user, changes) {
  user.name = changes.name;  // Mutates the original object
  user.email = changes.email;
  return user;
}

let config = { port: 3000 };
config.port = 8080;  // Mutable global state
```

## Good

```js
// Immutable — creates new copies
function addToCart(cart, item) {
  return [...cart, item];  // New array
}

function updateUser(user, changes) {
  return { ...user, ...changes };  // New object
}

const config = Object.freeze({ port: 3000 });
// config.port = 8080;  // Throws in strict mode, silently fails otherwise
```

## Deep Immutability

```js
// Shallow freeze — nested objects are still mutable
const settings = Object.freeze({
  theme: { primary: 'blue' },
});
// settings.theme.primary = 'red';  // Works — Object.freeze is shallow

// Deep freeze utility
function deepFreeze(obj) {
  Object.keys(obj).forEach(key => {
    if (typeof obj[key] === 'object' && obj[key] !== null && !Object.isFrozen(obj[key])) {
      deepFreeze(obj[key]);
    }
  });
  return Object.freeze(obj);
}
```

## Immutable Array Methods (ES2023+)

```js
const arr = [3, 1, 2];

// Old — mutates
arr.sort();         // Mutates original
arr.reverse();      // Mutates original
arr.splice(1, 1);   // Mutates original

// New — returns new array
const sorted = arr.toSorted();     // [1, 2, 3]
const reversed = arr.toReversed(); // [2, 1, 3]
const spliced = arr.toSpliced(1, 1); // [3, 2]
const replaced = arr.with(0, 99);  // [99, 1, 2]
```

## When Exceptions Apply

Mutation is acceptable in performance-critical hot paths where allocation overhead dominates, or in tight loops with large data structures. Always benchmark first.

## See Also

- [fn-pure-functions](./fn-pure-functions.md) - Write pure functions
- [fn-rest-spread](./fn-rest-spread.md) - Rest/spread operators
- [sec-prototype-pollution](./sec-prototype-pollution.md) - Guard against prototype pollution
