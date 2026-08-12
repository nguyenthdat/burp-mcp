# test-avoid-snapshot-abuse

> Use snapshot tests sparingly, and review generated snapshots deliberately

## Why It Matters

Snapshot tests are cheap to write and expensive to trust: a single `toMatchSnapshot()` on a large object silently accepts any change once someone runs `--update` without reading the diff, turning the test into a rubber stamp instead of a specification. Overused snapshots produce huge, unreviewable diffs in pull requests and hide real regressions inside noise (timestamps, generated IDs, whitespace). Snapshots earn their keep only for small, stable outputs where a human will actually read the diff before approving the update.

## Bad

```typescript
import { expect, test } from "vitest";
import { renderInvoice } from "./invoice";

test("renders invoice", () => {
  // A giant object snapshot nobody will read line-by-line on update
  const invoice = renderInvoice({ id: "inv_1", items: [/* 40 line items */] });
  expect(invoice).toMatchSnapshot();
});
```

## Good

```typescript
import { expect, test } from "vitest";
import { renderInvoiceHeader } from "./invoice";

test("should render the invoice header with formatted currency", () => {
  const header = renderInvoiceHeader({ id: "inv_1", total: 4999, currency: "USD" });

  // Explicit assertions on the fields that matter
  expect(header).toEqual({
    id: "inv_1",
    displayTotal: "$49.99",
    currency: "USD",
  });
});

test("should match the small, stable header markup", () => {
  const html = renderInvoiceHeader({ id: "inv_1", total: 4999, currency: "USD" }).toHtml();
  expect(html).toMatchInlineSnapshot(
    `"<h1>Invoice inv_1</h1><span>$49.99</span>"`,
  );
});
```

## When Snapshots Are Acceptable

- Small, deterministic outputs: a rendered component's DOM for a fixed set of props, a formatted CLI help string, a serialized error message.
- Prefer `toMatchInlineSnapshot()` over `toMatchSnapshot()` for small values — the expected output lives in the test file itself, so reviewers see the exact diff in the PR instead of a separate `.snap` file.
- Never snapshot values containing timestamps, UUIDs, or non-deterministic ordering without normalizing them first (e.g. replace `Date.now()` output with a placeholder before snapshotting).
- Treat any `--update-snapshots` run as a change that requires the same scrutiny as a manually written assertion change — review the diff, don't just accept it.

## See Also

- [test-coverage-meaningful](test-coverage-meaningful.md) - Target meaningful coverage of behavior, not a 100% coverage vanity metric
- [test-arrange-act-assert](test-arrange-act-assert.md) - Structure tests as arrange/act/assert
- [test-fake-timers](test-fake-timers.md) - Use fake timers to test time-dependent code deterministically
