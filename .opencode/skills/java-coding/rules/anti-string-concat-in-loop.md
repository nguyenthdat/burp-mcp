# anti-string-concat-in-loop

> Don't concatenate strings with `+` in a loop

## Why It Matters

`String` is immutable, so every `+` inside a loop allocates a brand-new `String` (and, depending on the JIT, may or may not be optimized into a `StringBuilder`) - across thousands of iterations that means thousands of throwaway allocations and O(n^2) total copying. `StringBuilder` mutates a single growable buffer, turning that into O(n) work and dramatically less garbage-collector pressure.

## Bad

```java
public String buildCsvRow(List<String> columns) {
  String row = "";
  for (String column : columns) {
    row = row + column + ","; // New String object allocated on every iteration
  }
  return row;
}

// With 10,000 rows of 20 columns each, this creates ~200,000 intermediate
// String objects, most of them immediately garbage
public String buildReport(List<Row> rows) {
  String report = "";
  for (Row row : rows) {
    report += row.toString() + "\n";
  }
  return report;
}
```

## Good

```java
public String buildCsvRow(List<String> columns) {
  StringBuilder row = new StringBuilder();
  for (String column : columns) {
    row.append(column).append(',');
  }
  return row.toString();
}

public String buildReport(List<Row> rows) {
  StringBuilder report = new StringBuilder(rows.size() * 64); // Pre-size when the count is known
  for (Row row : rows) {
    report.append(row).append('\n');
  }
  return report.toString();
}
```

## Even Better for Simple Joins

```java
// String.join and Collectors.joining avoid the manual StringBuilder entirely
public String buildCsvRow(List<String> columns) {
  return String.join(",", columns);
}

public String buildReport(List<Row> rows) {
  return rows.stream()
      .map(Row::toString)
      .collect(Collectors.joining("\n"));
}
```

## When Plain `+` Is Fine

```java
// A single, non-looped concatenation compiles to an efficient StringBuilder
// under the hood - there's no performance issue here.
String message = "User " + userId + " not found";

// Concatenating a small, fixed number of values outside a loop is fine too.
String fullName = firstName + " " + lastName;
```

## See Also

- [`perf-stringbuilder-loop-concat`](perf-stringbuilder-loop-concat.md) - The positive rule this anti-pattern violates
- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Using streams/collectors for building strings from collections
- [`lint-error-prone-compiler-plugin`](lint-error-prone-compiler-plugin.md) - Can flag some inefficient concatenation patterns at compile time
