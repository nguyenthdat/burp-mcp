# err-never-swallow

> Never silently swallow errors in empty catch blocks

## Why It Matters

An empty `catch {}` block (or one that only logs at a level nobody watches) makes failures invisible: the operation didn't succeed, but the program continues as if it did, corrupting downstream state or masking a bug that will be far harder to diagnose once its symptoms show up somewhere unrelated. Every catch block should do at least one of: handle the error meaningfully, log it with enough context to act on, or rethrow it — silence is never the right default.

## Bad

```typescript
async function syncInventory() {
  try {
    await pushUpdatesToWarehouse();
  } catch {
    // Silently ignored — nobody knows the sync failed until stock is wrong
  }
}

function parseMaybeJson(input: string) {
  try {
    return JSON.parse(input);
  } catch (e) {
    // Swallowed and returns undefined with no signal of why
  }
  return undefined;
}
```

## Good

```typescript
async function syncInventory() {
  try {
    await pushUpdatesToWarehouse();
  } catch (err) {
    logger.error("inventory sync failed", { error: err });
    throw new Error("inventory sync failed", { cause: err }); // caller decides what to do next
  }
}

function parseMaybeJson(input: string): unknown | undefined {
  try {
    return JSON.parse(input);
  } catch (err) {
    logger.warn("failed to parse JSON, falling back to undefined", {
      input,
      error: err instanceof Error ? err.message : String(err),
    });
    return undefined; // explicit, logged fallback — not a silent swallow
  }
}
```

## The Line Between "Handled" and "Swallowed"

Not every catch needs to rethrow — returning a fallback value is fine *as long as the failure is observable* (logged, counted in a metric, surfaced in a health check). The test is: if this exact failure started happening on every request tomorrow, would anyone notice?

| Pattern | Verdict |
|---|---|
| `catch {}` | Always wrong |
| `catch (e) { console.log(e) }` in production, no monitoring on console output | Effectively swallowed |
| `catch (e) { logger.error(...); return fallback }` | Acceptable — observable, has a deliberate fallback |
| `catch (e) { logger.error(...); throw }` | Acceptable — observable, propagates |

## Configuration

```json
{
  "rules": {
    "no-empty": ["error", { "allowEmptyCatch": false }]
  }
}
```

## See Also

- [err-specific-catch](err-specific-catch.md) - Catch and handle specific error types instead of a blanket catch-all
- [err-rethrow-context](err-rethrow-context.md) - Add context when rethrowing instead of losing the original error
- [anti-empty-catch-block](anti-empty-catch-block.md) - Avoid empty catch blocks that hide failures
