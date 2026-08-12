# perf-string-format-vs-concat

> Choose `String.format`/concatenation deliberately by cost

## Why It Matters

`String.format` and `MessageFormat` parse a format string at runtime, box every primitive argument, and use locale-aware machinery - convenient, but roughly an order of magnitude slower than direct concatenation or `StringBuilder` for simple cases. Using it reflexively for every log line or hot-path message adds up. The right choice depends on whether you need locale/number formatting (`String.format`), simple gluing (concatenation or `StringBuilder`), or a logging framework's own placeholder syntax (which defers formatting entirely).

## Bad

```java
private static final Logger log = LoggerFactory.getLogger(OrderService.class);

public void logOrder(Order order) {
    // Runs String.format's parsing/boxing cost even when DEBUG is disabled,
    // and even though no locale-sensitive formatting is needed.
    log.debug(String.format("Processing order %s for customer %s, total=%f",
        order.id(), order.customerId(), order.total()));
}

public String buildKey(String prefix, int index) {
    return String.format("%s-%d", prefix, index);  // Overkill for a simple join
}
```

## Good

```java
public void logOrder(Order order) {
    // Placeholders are only formatted if the log level is actually enabled
    log.debug("Processing order {} for customer {}, total={}",
        order.id(), order.customerId(), order.total());
}

public String buildKey(String prefix, int index) {
    return prefix + "-" + index;  // Single concatenation, compiled to one StringBuilder
}
```

## When `String.format` Is the Right Tool

```java
// Locale-sensitive currency formatting genuinely needs String.format/DecimalFormat -
// concatenation can't express grouping separators, padding, or precision this way.
String receipt = String.format(Locale.US, "Total: $%,.2f", amount);

// Fixed-width columnar output for a report benefits from format's padding specifiers.
String row = String.format("%-20s %10.2f%n", item.name(), item.price());
```

## See Also

- [`perf-stringbuilder-loop-concat`](perf-stringbuilder-loop-concat.md) - Use `StringBuilder` for string concatenation in loops
- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`modern-text-blocks`](modern-text-blocks.md) - Use text blocks for multi-line string literals
