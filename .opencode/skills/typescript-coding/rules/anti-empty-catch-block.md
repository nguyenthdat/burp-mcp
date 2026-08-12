# anti-empty-catch-block

> Don't leave catch blocks empty

## Why It Matters

An empty `catch {}` (or `catch (e) {}`) swallows every error a `try` block can throw with no trace whatsoever — the failure simply vanishes, and whoever eventually debugs the resulting incorrect behavior has no log line, no stack trace, and no indication that an exception was ever thrown in the first place. This is often introduced defensively ("just make the error go away so the app doesn't crash") but it converts a loud, debuggable failure into a silent, much harder to diagnose one. Every catch block should do something with the error: log it, rethrow it, convert it into a typed result, or — if truly intentional — document why swallowing is safe here.

## Bad

```typescript
async function loadUserPreferences(userId: string) {
  try {
    return await db.preferences.findOne({ userId });
  } catch {
    // Silently returns undefined on ANY failure: DB down, bad query, network blip
  }
}

function parseConfig(raw: string) {
  try {
    return JSON.parse(raw);
  } catch (e) {} // swallow parse errors entirely — caller gets `undefined`, no idea why
}
```

## Good

```typescript
async function loadUserPreferences(userId: string) {
  try {
    return await db.preferences.findOne({ userId });
  } catch (err) {
    logger.error({ err, userId }, 'failed to load user preferences');
    throw err; // or return a typed default, but never silently swallow
  }
}

function parseConfig(raw: string): Config | null {
  try {
    return JSON.parse(raw);
  } catch (err) {
    logger.warn({ err }, 'invalid config JSON, falling back to defaults');
    return null; // explicit, documented fallback — not a silent one
  }
}
```

## Genuinely Intentional Ignoring

If ignoring truly is correct (e.g., a best-effort cleanup operation whose failure doesn't matter), say so explicitly rather than leaving the block empty:

```typescript
try {
  await tempFile.cleanup();
} catch {
  // Intentionally ignored: cleanup failures don't affect correctness,
  // and the OS reclaims temp files on process exit regardless.
}
```

Enforce with ESLint's `no-empty` rule (which flags empty blocks generally) combined with code review attention to `catch` specifically, since `no-empty` alone allows a `catch` with only a comment.

## See Also

- [err-never-swallow](err-never-swallow.md) - Never swallow errors silently
- [err-specific-catch](err-specific-catch.md) - Catch specific error types, not everything
- [node-structured-logging](node-structured-logging.md) - Use structured, leveled logging instead of `console.log`
