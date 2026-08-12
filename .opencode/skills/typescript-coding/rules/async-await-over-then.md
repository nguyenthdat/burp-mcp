# async-await-over-then

> Prefer `async`/`await` over chained `.then()` calls

## Why It Matters

Chained `.then()` calls nest error handling awkwardly and make control flow (branches, loops, try/catch) much harder to express than in synchronous-looking code. `async`/`await` lets you use ordinary language constructs — `if`, `for`, `try/catch` — around asynchronous operations, which reduces bugs from misplaced `.catch()` handlers and forgotten `return` statements inside chains. Stack traces from `await` also point more reliably at the failing call site than deeply chained promise callbacks.

## Bad

```typescript
function loadUserProfile(id: string): Promise<Profile> {
  return fetchUser(id)
    .then((user) => {
      return fetchPreferences(user.id).then((prefs) => {
        if (prefs.theme === "legacy") {
          return fetchLegacyTheme().then((theme) => ({ user, prefs, theme }));
        }
        return { user, prefs, theme: prefs.theme };
      });
    })
    .catch((err) => {
      console.error("failed to load profile", err);
      throw err;
    });
}
```

## Good

```typescript
async function loadUserProfile(id: string): Promise<Profile> {
  try {
    const user = await fetchUser(id);
    const prefs = await fetchPreferences(user.id);
    const theme = prefs.theme === "legacy" ? await fetchLegacyTheme() : prefs.theme;
    return { user, prefs, theme };
  } catch (err) {
    console.error("failed to load profile", err);
    throw err;
  }
}
```

## When `.then()` Is Still Reasonable

- Short, single-step transformations on a promise you don't otherwise need to `await`, e.g. `fetch(url).then((r) => r.json())` returned directly from a function.
- Library code that must run in non-async contexts (older build targets without async transform support).
- Attaching a one-off side effect without blocking the calling function: `job.then(() => metrics.increment("done"))`.

In all other cases involving branching, multiple sequential steps, or non-trivial error handling, `async`/`await` produces more readable and correct code.

## See Also

- [async-no-floating-promises](async-no-floating-promises.md) - Never leave a promise floating; await, return, or explicitly void it
- [err-async-propagation](err-async-propagation.md) - Propagate errors correctly through async call chains
- [async-immediately-invoked](async-immediately-invoked.md) - Use an async IIFE to run async code in non-async contexts
