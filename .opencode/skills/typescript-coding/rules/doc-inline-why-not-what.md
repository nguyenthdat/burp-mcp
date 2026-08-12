# doc-inline-why-not-what

> Write comments that explain why, not what the code already says

## Why It Matters

A comment that restates the code in English adds a second thing that can go stale — when the code changes and the comment doesn't, the comment now actively lies, and readers learn to distrust (and eventually ignore) all comments in the file. Comments earn their keep by capturing information the code *can't* express: the business reason for a weird-looking check, a workaround for a specific bug in a dependency, or a decision that was deliberately made against the "obvious" alternative.

## Bad

```typescript
// Loop through all users
for (const user of users) {
  // if user is active
  if (user.status === "active") {
    // add user to result
    result.push(user);
  }
}

// Increment retry count by 1
retryCount += 1;
```

## Good

```typescript
for (const user of users) {
  if (user.status === "active") {
    result.push(user);
  }
}

// Stripe's webhook retries can arrive out of order up to 3 days after
// the original event, so we track attempts per-event to detect and
// discard duplicates rather than trusting delivery order.
retryCount += 1;
```

```typescript
// Deliberately not using Array.prototype.sort() here: V8's sort became
// stable in Node 11, but the polyfilled environment this bundle also
// targets (see rollup.config.js) still ships an unstable sort.
const sorted = stableSort(items, (a, b) => a.priority - b.priority);
```

## Where "Why" Comments Add Value

- Explaining a non-obvious business rule: "orders under $5 skip fraud review per finance policy FIN-114".
- Documenting a workaround: "Safari 15 fires `resize` twice on rotation; debounce absorbs the duplicate — remove once caniuse shows Safari 15 usage below 1%."
- Recording a rejected alternative: "not using a Map here because insertion order matters for the UI and we need `Object.entries()` ordering guarantees."
- Flagging a deliberate deviation from a lint rule or convention, right above the `// eslint-disable-next-line` that suppresses it.

## See Also

- [doc-type-as-documentation](doc-type-as-documentation.md) - Let precise types replace comments that only describe a shape
- [doc-tsdoc-public-api](doc-tsdoc-public-api.md) - Document all public API with TSDoc comments
- [anti-magic-numbers](anti-magic-numbers.md) - name constants instead of leaving unexplained literals that need a comment
