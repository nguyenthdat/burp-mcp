# perf-prepare-regex

> Compile regular expressions once outside of loops and hot functions

## Why It Matters

Creating a `RegExp` object inside a loop or frequently-called function recompiles the pattern each time, wasting CPU. JavaScript engines cache regex literals, but the `RegExp()` constructor and dynamically-created regexes are not cached. Defining regex at module scope ensures it's compiled once and reused.

## Bad

```js
// Recompiled on every call
function isValidEmail(email) {
  const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return regex.test(email);
}

// Dynamic regex inside loop — recompiled each iteration
function findMatches(text, patterns) {
  return patterns.map(pattern => {
    const regex = new RegExp(pattern, 'i');  // Recompiled per pattern
    return text.match(regex);
  });
}
```

## Good

```js
// Compiled once at module scope
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

function isValidEmail(email) {
  return EMAIL_REGEX.test(email);
}

// Pre-compile dynamic patterns
function createPatternMatcher(patterns) {
  const regexes = patterns.map(p => new RegExp(p, 'i'));  // Compile once
  return (text) => regexes.map(re => text.match(re));
}

const match = createPatternMatcher(['foo', 'bar']);
match('hello foo world');  // No regex compilation
```

## When Exceptions Apply

Regex literals (`/pattern/flags`) are cached by the engine, so defining them inside a function is fine. The `RegExp()` constructor and `new RegExp()` are not cached and should be hoisted.

## See Also

- [sec-regex-dos](./sec-regex-dos.md) - Avoid ReDoS
- [perf-memoize](./perf-memoize.md) - Memoize expensive functions
