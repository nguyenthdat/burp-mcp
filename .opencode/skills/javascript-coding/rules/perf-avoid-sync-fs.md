# perf-avoid-sync-fs

> Never use synchronous fs/path operations inside request handlers or event loop callbacks

## Why It Matters

Synchronous filesystem operations (`readFileSync`, `existsSync`, `statSync`) block the event loop completely. In a server handling 100 concurrent requests, a single 10ms synchronous read blocks ALL requests for that duration. This cascades into latency spikes and poor throughput. Use `fs/promises` or callbacks instead.

## Bad

```js
// Blocks all concurrent requests
app.get('/file', (req, res) => {
  const data = readFileSync('./data.txt', 'utf8');  // Blocks!
  res.send(data);
});

app.use((req, res, next) => {
  if (existsSync('./maintenance.lock')) {  // Blocks!
    return res.status(503).send('Maintenance mode');
  }
  next();
});
```

## Good

```js
// Non-blocking async operations
app.get('/file', async (req, res) => {
  const data = await readFile('./data.txt', 'utf8');
  res.send(data);
});

app.use(async (req, res, next) => {
  try {
    await access('./maintenance.lock');
    return res.status(503).send('Maintenance mode');
  } catch {
    next();
  }
});
```

## When Exceptions Apply

Synchronous operations are acceptable during startup (before the server listens) and in CLI scripts. Once the server is accepting requests, all I/O must be async.

## See Also

- [node-fs-async-over-sync](./node-fs-async-over-sync.md) - Use fs/promises
- [async-no-blocking-loop](./async-no-blocking-loop.md) - Don't block the event loop
