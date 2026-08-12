# fn-avoid-side-effects-in-map

> Never use `.map()` purely for side effects; use `.forEach()` or a `for` loop

## Why It Matters

`.map()` communicates a specific promise to every reader: "this produces a new array by transforming each element, and I care about the result." Calling `.map()` and discarding its return value — using it only to trigger side effects like logging, writing to a database, or mutating something external — breaks that promise, forces the reader to check whether the return value matters (it doesn't), and wastes memory building an array that's immediately thrown away. It also causes a specific and common bug with `async` callbacks: `array.map(async (x) => ...)` produces an array of `Promise`s that nobody awaits.

## Bad

```typescript
// Return value is discarded — `.map()` here is just a confusing `.forEach()`
users.map((user) => {
  console.log(`Processing ${user.name}`);
  saveToDatabase(user);
});

// Classic async trap: map's callback is async, but map itself is NOT async-aware
async function notifyAll(users: User[]) {
  users.map(async (user) => {
    await sendEmail(user.email); // fires, but nothing waits for it
  });
  console.log("All notified"); // logs immediately, before any email actually sends
}
```

## Good

```typescript
users.forEach((user) => {
  console.log(`Processing ${user.name}`);
  saveToDatabase(user);
});

async function notifyAll(users: User[]) {
  await Promise.all(users.map((user) => sendEmail(user.email)));
  console.log("All notified"); // only logs after every email has actually sent
}
```

Note the async fix still uses `.map()` — but here it correctly produces an array of `Promise<void>` that `Promise.all` then awaits. The rule isn't "never combine map with async," it's "don't call `.map()` for side effects and ignore what it returns."

## Quick Test

Ask: "if I deleted the variable capturing this call's return value (or never had one), would anything change?" If the answer is no, and you never use the return value, you're using `.map()` as a `.forEach()` with extra allocation — switch to `.forEach()` or a `for...of` loop.

## Lint Enforcement

```jsonc
{
  "rules": {
    "array-callback-return": ["error", { "checkForEach": false }],
    "@typescript-eslint/no-floating-promises": "error"
  }
}
```

`no-floating-promises` (with `checkThenables`) is particularly good at catching the `array.map(async ...)` pattern above, since the resulting `Promise<void>[]` return value being ignored is exactly what it flags.

## See Also

- [fn-array-methods-over-loops](fn-array-methods-over-loops.md) - Use `map`/`filter`/`reduce` for transformations instead of manual `for` loops
- [async-avoid-async-foreach](async-avoid-async-foreach.md) - `forEach` does not await its callback; use `for...of` for sequential async work
- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a `Promise` unhandled; await it, return it, or explicitly void it
