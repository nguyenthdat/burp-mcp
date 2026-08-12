# err-try-catch-narrow

> Keep try blocks as small as possible — only wrap the code that can throw

## Why It Matters

Large try blocks catch errors from code you didn't intend to handle, masking bugs. If a line of code unexpectedly throws inside a broad try block, it gets caught as if it were an expected failure, silently bypassing your error recovery logic. Narrow try blocks make it clear exactly which operation might fail.

## Bad

```js
// Catches errors from process() too — masks bugs
try {
  const data = await fetchData();
  const parsed = JSON.parse(data);
  const result = process(parsed);   // Bug here gets caught silently
  await save(result);
  return result;
} catch (err) {
  console.error('Operation failed:', err);
  return null;
}
```

## Good

```js
// Only wrap the specific operations that can fail
let data;
try {
  data = await fetchData();
} catch (err) {
  console.error('Fetch failed:', err);
  return null;
}

let parsed;
try {
  parsed = JSON.parse(data);
} catch (err) {
  console.error('Parse failed:', err);
  return null;
}

// process() and save() errors bubble up — they're bugs
const result = process(parsed);
await save(result);
return result;
```

## Extracting to Functions

```js
async function loadData() {
  const data = await fetchWithErrorHandling();
  if (!data) return null;

  const parsed = parseWithErrorHandling(data);
  if (!parsed) return null;

  return parsed;
}

async function fetchWithErrorHandling() {
  try {
    return await fetchData();
  } catch (err) {
    console.error('Fetch failed:', err);
    return null;
  }
}

function parseWithErrorHandling(data) {
  try {
    return JSON.parse(data);
  } catch (err) {
    console.error('Parse failed:', err);
    return null;
  }
}
```

## When Exceptions Apply

In top-level request handlers, a broader try/catch is acceptable as a last-resort safety net:

```js
app.get('/api/data', async (req, res) => {
  try {
    const result = await loadAndProcess(req.query);
    res.json(result);
  } catch (err) {
    console.error('Unhandled error in /api/data:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});
```

## See Also

- [err-finally-cleanup](./err-finally-cleanup.md) - Use finally for cleanup
- [err-async-await-catch](./err-async-await-catch.md) - try/catch with await
