# anti-callback-hell

> Don't nest callbacks; use async/await instead

## Why It Matters

Nesting asynchronous callbacks — the classic "pyramid of doom" — forces error handling to be duplicated at every level (each callback gets its own `if (err)` branch), makes control flow hard to follow because execution order doesn't match the visual top-to-bottom order once error paths are involved, and makes a single missing `return` after an error check silently continue execution down the success path. `async`/`await`, backed by Promises, lets asynchronous code read top-to-bottom like synchronous code, centralizes error handling in one `try`/`catch`, and composes naturally with `Promise.all`/`Promise.allSettled` for concurrent work — all without changing the underlying non-blocking model.

## Bad

```typescript
function loadUserDashboard(userId: string, callback: (err: Error | null, data?: Dashboard) => void) {
  getUser(userId, (err, user) => {
    if (err) return callback(err);
    getOrders(user.id, (err, orders) => {
      if (err) return callback(err);
      getRecommendations(user.id, (err, recs) => {
        if (err) return callback(err);
        // Four levels deep, error handling duplicated at each level
        callback(null, { user, orders, recommendations: recs });
      });
    });
  });
}
```

## Good

```typescript
async function loadUserDashboard(userId: string): Promise<Dashboard> {
  const user = await getUser(userId);
  const [orders, recommendations] = await Promise.all([
    getOrders(user.id),
    getRecommendations(user.id),
  ]);
  return { user, orders, recommendations };
  // A single try/catch at the call site handles every failure in this chain
}

// Call site
try {
  const dashboard = await loadUserDashboard(userId);
  render(dashboard);
} catch (err) {
  logger.error({ err }, 'failed to load dashboard');
  renderErrorState();
}
```

## Migrating Callback-Based APIs

For legacy Node.js APIs that still use the `(err, result)` callback style, `util.promisify` bridges them into the `async`/`await` world instead of hand-nesting:

```typescript
import { promisify } from 'node:util';
import { readFile } from 'node:fs';

const readFileAsync = promisify(readFile);
const contents = await readFileAsync('./data.json', 'utf8');
```

## See Also

- [async-await-over-then](async-await-over-then.md) - Prefer `async`/`await` over chained `.then()` calls
- [async-promise-all-parallel](async-promise-all-parallel.md) - Run independent async operations in parallel with `Promise.all`
- [err-async-propagation](err-async-propagation.md) - Let async errors propagate through rejection, not callbacks
