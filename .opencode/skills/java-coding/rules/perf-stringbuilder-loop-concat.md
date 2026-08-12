# perf-stringbuilder-loop-concat

> Use `StringBuilder` for string concatenation in loops

## Why It Matters

`String` is immutable in Java, so every `+` concatenation inside a loop allocates a brand-new `String` and copies all prior characters into it, turning what looks like O(n) work into O(n^2). For a loop building a report over a few thousand rows, this can mean millions of wasted character copies and constant garbage-collector pressure. `StringBuilder` amortizes growth with a resizable backing array, making the same loop O(n).

## Bad

```java
public String buildCsvRow(List<String> columns) {
    String row = "";
    for (String column : columns) {
        row = row + column + ",";  // New String + copy of everything so far, every iteration
    }
    return row;
}

public String renderReport(List<Order> orders) {
    String report = "Order Report\n";
    for (Order order : orders) {
        report += order.id() + ": " + order.total() + "\n";  // Compiler inserts a fresh StringBuilder per iteration
    }
    return report;
}
```

## Good

```java
public String buildCsvRow(List<String> columns) {
    StringBuilder sb = new StringBuilder();
    for (String column : columns) {
        sb.append(column).append(',');
    }
    return sb.toString();
}

public String renderReport(List<Order> orders) {
    StringBuilder sb = new StringBuilder(orders.size() * 32);  // Pre-size using known count
    sb.append("Order Report\n");
    for (Order order : orders) {
        sb.append(order.id()).append(": ").append(order.total()).append('\n');
    }
    return sb.toString();
}
```

## When Plain Concatenation Is Fine

```java
// A single, non-looped concatenation is compiled into one StringBuilder
// by javac already - no need to hand-roll it.
String message = "User " + userId + " not found";

// String.join is clearer than a manual loop for simple delimiter-joining.
String csv = String.join(",", columns);
```

## See Also

- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`perf-string-format-vs-concat`](perf-string-format-vs-concat.md) - Choose `String.format`/concatenation deliberately by cost
- [`perf-collection-sizing`](perf-collection-sizing.md) - Size collections up front when the count is known
- [`anti-string-concat-in-loop`](anti-string-concat-in-loop.md) - Anti-pattern reference for loop concatenation
