# async-microtask-queue

> Understand microtask vs macrotask ordering to avoid subtle timing bugs

## Why It Matters

JavaScript has two queues: microtasks (Promise callbacks, queueMicrotask) and macrotasks (setTimeout, setInterval, I/O). Microtasks execute before the next macrotask. Misunderstanding this ordering leads to race conditions, missing values, and UI rendering glitches. Knowing the execution order is essential for corret debouncing, batching, and state management.

## Bad

```js
// Expecting setTimeout to run before promise resolution
let value = null;

setTimeout(() => {
  value = 'timeout';
}, 0);

Promise.resolve().then(() => {
  value = 'microtask';
});

console.log(value);  // null — neither has run yet

// After sync code finishes: value is 'microtask' (microtask ran first)
// timeout fires later
```

## Good

```js
// Correct understanding of execution order
console.log('1. sync');

setTimeout(() => console.log('4. macrotask (setTimeout)'), 0);

queueMicrotask(() => console.log('3. microtask (queueMicrotask)'));

Promise.resolve().then(() => console.log('2. microtask (Promise.then)'));

// Output order:
// 1. sync
// 2. microtask (Promise.then)
// 3. microtask (queueMicrotask)
// 4. macrotask (setTimeout)
```

## Breaking up Long Tasks

Use `setTimeout` to yield to the event loop and avoid blocking:

```js
function processLargeArray(items) {
  return new Promise((resolve) => {
    let i = 0;

    function chunk() {
      const end = Math.min(i + 1000, items.length);
      for (; i < end; i++) {
        doWork(items[i]);
      }
      if (i < items.length) {
        setTimeout(chunk, 0);  // Yield to event loop between chunks
      } else {
        resolve();
      }
    }

    chunk();
  });
}
```

## When Exceptions Apply

In server-side Node.js code without a browser rendering pipeline, microtask vs macrotask differences are less visible. But they still matter for I/O ordering and timer accuracy.

## See Also

- [async-avoid-floating-promises](./async-avoid-floating-promises.md) - Promise lifecycle
- [perf-debounce-throttle](./perf-debounce-throttle.md) - Event frequency control
