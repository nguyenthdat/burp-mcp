# err-rethrow-context

> Add context when rethrowing instead of losing the original error

## Why It Matters

Rethrowing the exact error you caught, unchanged, means every layer of the call stack that touches it contributes nothing to debugging — you end up with a stack trace pointing at a low-level file read or SQL query, with no indication of *which* higher-level operation was in progress when it failed. Wrapping the error with additional context at each layer (what was being attempted, with what inputs) turns a bare "ENOENT" into "failed to load user avatar for user 42: ENOENT," without discarding the original error via `cause`.

## Bad

```typescript
async function loadUserAvatar(userId: string) {
  const path = `/avatars/${userId}.png`;
  return fs.readFile(path); // if this throws ENOENT, the error says nothing about userId or intent
}

async function renderProfile(userId: string) {
  try {
    const avatar = await loadUserAvatar(userId);
    return render(avatar);
  } catch (err) {
    throw err; // rethrown as-is — no added context, and this line is a no-op anyway
  }
}
```

## Good

```typescript
async function loadUserAvatar(userId: string) {
  const path = `/avatars/${userId}.png`;
  try {
    return await fs.readFile(path);
  } catch (err) {
    throw new Error(`failed to load avatar for user ${userId} at ${path}`, { cause: err });
  }
}

async function renderProfile(userId: string) {
  try {
    const avatar = await loadUserAvatar(userId);
    return render(avatar);
  } catch (err) {
    throw new Error(`failed to render profile for user ${userId}`, { cause: err });
  }
}

// Resulting error, when logged, shows the full chain:
// Error: failed to render profile for user 42
//   caused by: Error: failed to load avatar for user 42 at /avatars/42.png
//     caused by: Error: ENOENT: no such file or directory, open '/avatars/42.png'
```

## Don't Add Context You Don't Have

If a catch block has no additional information to contribute beyond what the original error already carries, don't wrap it — either let it propagate un-caught, or catch it specifically to handle it (see `err-specific-catch`). Wrapping indiscriminately at every layer produces noisy, redundant messages that bury the useful part.

```typescript
// Unnecessary wrapping — adds no information the original error didn't have
async function getUser(id: string) {
  try {
    return await db.query("SELECT * FROM users WHERE id = ?", [id]);
  } catch (err) {
    throw new Error("query failed", { cause: err }); // "query failed" tells you nothing new
  }
}
```

## See Also

- [err-cause-chaining](err-cause-chaining.md) - Chain root causes with the standard cause option
- [err-custom-error-class](err-custom-error-class.md) - Extend Error with custom subclasses that carry structured context
- [err-specific-catch](err-specific-catch.md) - Catch and handle specific error types instead of a blanket catch-all
