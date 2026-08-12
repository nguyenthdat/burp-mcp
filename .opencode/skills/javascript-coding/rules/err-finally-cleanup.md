# err-finally-cleanup

> Use finally blocks for resource cleanup regardless of success or failure

## Why It Matters

Resources like file handles, database connections, and sockets must be released even when an operation fails. Without `finally`, cleanup code must be duplicated in both `try` and `catch` paths, or worse, skipped on errors. The `finally` block guarantees cleanup runs in all exit paths, including early returns and thrown exceptions.

## Bad

```js
// Cleanup duplicated — error-prone
async function readConfig(path) {
  const handle = await openFile(path);
  try {
    return await parseFile(handle);
  } catch (err) {
    await closeFile(handle);
    throw err;
  }
  // Missing: await closeFile(handle) in success path
}

// Manual cleanup — easy to forget
let client;
try {
  client = await db.connect();
  return await client.query('SELECT ...');
} catch (err) {
  if (client) await client.release();
  throw err;
}
```

## Good

```js
// finally ensures cleanup always runs
async function readConfig(path) {
  const handle = await openFile(path);
  try {
    return await parseFile(handle);
  } finally {
    await closeFile(handle);
  }
}

// With database connections
async function query(sql) {
  const client = await pool.connect();
  try {
    return await client.query(sql);
  } finally {
    client.release();  // Always runs, even on query error
  }
}
```

## Using AbortController for cleanup

```js
async function fetchWithCleanup(url, ms) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), ms);

  try {
    return await fetch(url, { signal: controller.signal });
  } finally {
    clearTimeout(timer);  // Always clean up the timer
  }
}
```

## When Exceptions Apply

`finally` is unnecessary when the resource is automatically cleaned up — e.g., `fs/promises` file handles close on garbage collection, and HTTP responses auto-end. But explicit cleanup is still preferred for clarity.

## See Also

- [err-try-catch-narrow](./err-try-catch-narrow.md) - Keep try blocks small
- [async-abort-control](./async-abort-control.md) - AbortController for cancellation
