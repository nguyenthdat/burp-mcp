# node-fs-async-over-sync

> Always use `fs/promises` over `fs` synchronous methods in async contexts

## Why It Matters

Synchronous filesystem operations (`readFileSync`, `writeFileSync`) block the event loop, freezing all I/O in your application. In a server, a single sync call during a request can degrade performance for all concurrent requests. `fs/promises` provides the same API as async/await-compatible functions that don't block.

## Bad

```js
// Blocks the event loop
import { readFileSync, writeFileSync } from 'node:fs';

app.get('/config', (req, res) => {
  const config = readFileSync('./config.json', 'utf8');
  res.json(JSON.parse(config));
});

app.post('/data', (req, res) => {
  writeFileSync('./data.json', JSON.stringify(req.body));
  res.json({ ok: true });
});
```

## Good

```js
// Non-blocking async operations
import { readFile, writeFile } from 'node:fs/promises';

app.get('/config', async (req, res) => {
  const config = await readFile('./config.json', 'utf8');
  res.json(JSON.parse(config));
});

app.post('/data', async (req, res) => {
  await writeFile('./data.json', JSON.stringify(req.body));
  res.json({ ok: true });
});
```

## When Sync Is Acceptable

```js
// At startup — before the server starts accepting requests
import { readFileSync } from 'node:fs';

const config = JSON.parse(readFileSync('./config.json', 'utf8'));

const server = createServer(app);
server.listen(config.port);

// In CLI scripts
#!/usr/bin/env node
import { readFileSync } from 'node:fs';
const input = readFileSync(process.argv[2], 'utf8');
```

## See Also

- [async-no-blocking-loop](./async-no-blocking-loop.md) - Don't block the event loop
- [perf-avoid-sync-fs](./perf-avoid-sync-fs.md) - Never sync fs in request handlers
