# perf-debounce-throttle

> Use debounce for events that should fire once after inactivity; throttle for events that should fire at most once per interval

## Why It Matters

Scroll, resize, keypress, and input events can fire hundreds of times per second. Running expensive operations on every event degrades responsiveness and wastes resources. Debounce waits for a pause in events; throttle limits the maximum frequency. Choosing the right one prevents jank without missing important updates.

## Bad

```js
// Expensive operation on every keystroke — janky
input.addEventListener('input', (e) => {
  fetch(`/search?q=${e.target.value}`).then(updateResults);
});

// On every scroll — kills performance
window.addEventListener('scroll', () => {
  updateScrollPosition();
  checkVisibility();
  animateElements();
});
```

## Good

```js
// Debounce — wait for 300ms pause in typing
function debounce(fn, delay) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
}

input.addEventListener('input', debounce(async (e) => {
  const results = await fetch(`/search?q=${e.target.value}`);
  updateResults(results);
}, 300));

// Throttle — at most once every 100ms
function throttle(fn, interval) {
  let lastTime = 0;
  return (...args) => {
    const now = Date.now();
    if (now - lastTime >= interval) {
      lastTime = now;
      fn(...args);
    }
  };
}

window.addEventListener('scroll', throttle(() => {
  updateScrollPosition();
}, 100));
```

## When to Use Each

| Pattern | Use Case | Example |
|---------|----------|---------|
| Debounce | Wait for activity to stop | Search-as-you-type, form validation, auto-save |
| Throttle | Cap frequency | Scroll handlers, resize handlers, game loop |
| requestAnimationFrame | Sync with repaint | Visual animations, DOM batch updates |

## See Also

- [fn-pure-functions](./fn-pure-functions.md) - Pure functions
- [async-abort-control](./async-abort-control.md) - Cancel previous requests
