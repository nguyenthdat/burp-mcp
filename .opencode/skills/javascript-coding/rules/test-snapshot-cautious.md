# test-snapshot-cautious

> Use snapshots sparingly — they're brittle and often reviewed carelessly

## Why It Matters

Snapshot tests compare output to a stored file. They're easy to create but dangerous: large snapshots are accepted without review, they break on any output change (even intentional ones), and developers habitually update them with `--update-snapshots` instead of diagnosing failures. Use snapshots only for small, stable outputs where the exact format is the contract.

## Bad

```js
// Large snapshot — any change breaks it, no one reviews the diff
test('renders dashboard', () => {
  const html = renderDashboard({
    users: 1000,
    orders: 5000,
    config: { /* large config */ },
  });
  // Snapshot is 500 lines — impossible to review
  assert.matchSnapshot(html);
});

// Snapshot of a random/date-dependent output
test('generates report', () => {
  const report = generateReport();  // Contains Date.now() — snapshot changes every run
  assert.matchSnapshot(report);
});
```

## Good

```js
// Inline snapshot for small, stable output
test('formats user name', () => {
  const result = formatName({ first: 'John', last: 'Doe' });
  assert.matchSnapshot(result);
});
// Snapshot file:
// `John Doe`

// Or: prefer explicit assertions over snapshots
test('formats user name', () => {
  const result = formatName({ first: 'John', last: 'Doe' });
  assert.strictEqual(result, 'John Doe');  // Clearer than a snapshot
});

// Snapshot is appropriate for serialized API responses (if stable)
test('serializes user to API format', () => {
  const user = { id: 1, name: 'Alice', email: 'alice@example.com' };
  const json = serializeUser(user);
  assert.matchSnapshot(json);  // Small, readable, stable
});
```

## When Exceptions Apply

Snapshots are appropriate for:
- Small (< 20 lines) output that represents a stable contract
- Error messages where exact wording matters
- Serialized data where exact format must be preserved

## See Also

- [test-fixture-factories](./test-fixture-factories.md) - Factory functions for test data
- [test-arrange-act-assert](./test-arrange-act-assert.md) - Test structure
