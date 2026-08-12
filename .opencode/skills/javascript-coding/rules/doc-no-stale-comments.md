# doc-no-stale-comments

> Remove commented-out code — use git history for old implementations

## Why It Matters

Commented-out code accumulates as developers are unsure whether to delete it. It rots (becomes incompatible with the current codebase), confuses readers ("is this code important?"), and adds noise. Git preserves every deleted line — code should live in version history, not in comments.

## Bad

```js
function processOrder(order) {
  // Old implementation before v2
  // const result = legacyProcess(order);
  // if (result.status === 'error') {
  //   return handleError(result);
  // }

  // Another approach (didn't work)
  // try {
  //   return await newApproach(order);
  // } catch {
  //   // fall through
  // }

  return newProcess(order);  // Current implementation
}
```

## Good

```js
function processOrder(order) {
  // Old implementation removed — see git history:
  // git log -p -- src/order-service.js
  return newProcess(order);
}

// If the old code documents a known approach that might be needed:
// Add a comment explaining the trade-off, not the code
function processOrder(order) {
  // Using sync processing instead of async because orders must
  // be processed sequentially to prevent race conditions on inventory.
  // See ADR #42 for the full decision.
  return newProcess(order);
}
```

## When Exceptions Apply

Temporarily commenting out code during active debugging is fine — but remove it before committing. Documentation examples and code snippets in JSDoc are intentional and not stale.

## See Also

- [doc-inline-comments-why](./doc-inline-comments-why.md) - Comment why, not what
- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
