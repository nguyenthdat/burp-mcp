# perf-string-concat-builder

> Build large strings with arrays/template literals, not repeated `+=` concatenation

## Why It Matters

Strings in JavaScript are immutable, so every `str += chunk` in a loop allocates a brand-new string and copies the previous contents into it; over thousands of iterations this becomes quadratic work (each concatenation copies an ever-growing string) rather than linear. Modern JS engines optimize simple cases with rope-like internal representations, but this optimization isn't guaranteed across engines or usage patterns, and collecting chunks in an array then joining once is both more predictable and, for genuinely large strings, meaningfully faster.

## Bad

```typescript
function buildCsvRow(fields: string[]): string {
  let row = "";
  for (const field of fields) {
    row += field + ","; // reallocates and copies the whole string each time
  }
  return row.slice(0, -1);
}

function buildLargeReport(rows: ReportRow[]): string {
  let report = "";
  for (const row of rows) {
    report += `${row.date},${row.amount},${row.description}\n`; // quadratic over many rows
  }
  return report;
}
```

## Good

```typescript
function buildCsvRow(fields: string[]): string {
  return fields.join(",");
}

function buildLargeReport(rows: ReportRow[]): string {
  const lines: string[] = [];
  for (const row of rows) {
    lines.push(`${row.date},${row.amount},${row.description}`);
  }
  return lines.join("\n");
}
```

## Guidelines

- Collect chunks in an array and call `.join(separator)` once at the end, rather than concatenating in a loop — this is the idiomatic pattern for building large strings (CSV rows, generated code, log output).
- Template literals are fine for a fixed, small number of interpolations (`` `${a}, ${b}` ``); the concern is specifically about *repeated* concatenation growing a string across many loop iterations.
- For streaming output (writing a large file or HTTP response), prefer writing chunks directly to a stream (`res.write(chunk)`) instead of building one giant string in memory and writing it all at once — this also avoids holding the entire output in memory.
- This is a genuine hot-path concern for reports, logs, and generated files with thousands+ of rows; for a handful of concatenations, `+=` is perfectly fine and more readable — don't rewrite trivial string building for a savings you can't measure (see `perf-avoid-premature-optimize`).

## See Also

- [perf-avoid-premature-optimize](perf-avoid-premature-optimize.md) - Profile before optimizing
- [node-streams-backpressure](node-streams-backpressure.md) - stream large output instead of buffering it all in memory
- [perf-avoid-unnecessary-allocation](perf-avoid-unnecessary-allocation.md) - Avoid allocating objects/arrays inside hot loops
