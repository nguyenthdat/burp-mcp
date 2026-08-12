# anti-sync-in-async

> Don't use synchronous I/O operations inside async request handlers

## Why It Matters

A single synchronous I/O call in an async request handler blocks the entire event loop for every other concurrent request. All other requests queued up or in flight must wait. This causes cascading latency spikes. The event loop is cooperative — one blocking call starves everyone.

## Bad

```js
app.get('/report', async (req, res) => {
  const template = readFileSync('./report.html', 'utf8');  // Blocks all requests!
  const data = await fetchData();
  const html = renderTemplate(template, data);
  res.send(html);
});
```

## Good

```js
app.get('/report', async (req, res) => {
  const [template, data] = await Promise.all([
    readFile('./report.html', 'utf8'),
    fetchData(),
  ]);
  const html = renderTemplate(template, data);
  res.send(html);
});
```

## When Exceptions Apply

Startup code (before the server listens) and CLI one-shot scripts can use sync I/O. Never in request handlers, middleware, or event loop callbacks.

## See Also

- [node-fs-async-over-sync](./node-fs-async-over-sync.md) - Use fs/promises
- [perf-avoid-sync-fs](./perf-avoid-sync-fs.md) - Never sync fs in handlers
