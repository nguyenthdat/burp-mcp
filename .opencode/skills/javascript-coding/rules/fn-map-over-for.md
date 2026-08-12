# fn-map-over-for

> Prefer .map(), .filter(), .reduce() and other array methods over imperative for loops

## Why It Matters

Array methods like `.map()`, `.filter()`, and `.reduce()` express the intent of the operation declaratively. They're chainable, don't leak temporary variables, and avoid off-by-one errors. Imperative `for` loops mix iteration logic with transformation logic, making code harder to scan and reason about.

## Bad

```js
// Imperative — intent is buried in loop mechanics
const activeNames = [];
for (let i = 0; i < users.length; i++) {
  if (users[i].isActive) {
    activeNames.push(users[i].name.toUpperCase());
  }
}

const total = 0;
for (let i = 0; i < orders.length; i++) {
  total += orders[i].amount;
}
```

## Good

```js
// Declarative — intent is clear
const activeNames = users
  .filter(user => user.isActive)
  .map(user => user.name.toUpperCase());

const total = orders.reduce((sum, order) => sum + order.amount, 0);
```

## Choosing the Right Method

```js
// .map() — transform each element
const ids = users.map(u => u.id);

// .filter() — keep elements that match condition
const adults = users.filter(u => u.age >= 18);

// .reduce() — aggregate to a single value
const grouped = users.reduce((acc, u) => {
  acc[u.role] = [...(acc[u.role] || []), u];
  return acc;
}, {});

// .find() — get first matching element
const admin = users.find(u => u.role === 'admin');

// .some() / .every() — test conditions
const hasAdmin = users.some(u => u.role === 'admin');
const allActive = users.every(u => u.isActive);

// .flatMap() — map and flatten
const tags = posts.flatMap(p => p.tags);
```

## When Exceptions Apply

Use `for...of` loops when:
- You need to `break` or `continue` based on complex logic
- Performance profiling shows array methods are the bottleneck (rare)
- You're working with very large arrays where `.reduce()` accumulator allocation matters
- You need `await` inside the loop (use `for...of`, not `forEach`)

## See Also

- [perf-for-of-array](./perf-for-of-array.md) - for...of performance
- [anti-await-loop](./anti-await-loop.md) - Don't await inside forEach
