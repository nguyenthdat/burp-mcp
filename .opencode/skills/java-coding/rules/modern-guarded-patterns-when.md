# modern-guarded-patterns-when

> Use guarded patterns (`when` clauses) in `switch`

## Why It Matters

Pattern matching for `switch` can select on type alone, but real-world logic often needs an additional condition on the matched value (e.g. "a Circle, but only if its radius is large"). Before guarded patterns, this required binding the pattern and then nesting an `if` inside the case body, which breaks the flat, exhaustive structure of the switch and can make exhaustiveness checking less precise. The `when` clause (finalized alongside record patterns in Java 21) attaches a boolean guard directly to a case label.

## Bad

```java
sealed interface Shape permits Circle, Rectangle {}
record Circle(double radius) implements Shape {}
record Rectangle(double width, double height) implements Shape {}

String classify(Shape shape) {
    return switch (shape) {
        case Circle c -> {
            // Guard logic buried inside the case body, breaking the flat structure
            if (c.radius() > 10) {
                yield "Large circle";
            } else {
                yield "Small circle";
            }
        }
        case Rectangle r -> "Rectangle";
    };
}
```

## Good

```java
String classify(Shape shape) {
    return switch (shape) {
        case Circle c when c.radius() > 10 -> "Large circle";
        case Circle c -> "Small circle";
        case Rectangle r when r.width() == r.height() -> "Square";
        case Rectangle r -> "Rectangle";
    };
}
```

## Guards Combine Naturally With Record Deconstruction

```java
record Point(int x, int y) {}

String locate(Object obj) {
    return switch (obj) {
        case Point(var x, var y) when x == 0 && y == 0 -> "Origin";
        case Point(var x, var y) when x == 0 -> "On the Y axis";
        case Point(var x, var y) when y == 0 -> "On the X axis";
        case Point p -> "At (%d, %d)".formatted(p.x(), p.y());
        default -> "Not a point";
    };
}
```

## Exhaustiveness With Guards

A guarded case (`when`) is never considered exhaustive on its own, because the compiler cannot statically evaluate the boolean condition - you must always follow it with an unguarded case for the same type (as `Circle c ->` does above) or a `default`, otherwise the switch fails to compile as non-exhaustive.

## See Also

- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - The base pattern-matching-in-switch mechanism this rule refines
- [`modern-record-deconstruction-patterns`](modern-record-deconstruction-patterns.md) - Deconstruction patterns commonly combined with guards
- [`modern-switch-expressions`](modern-switch-expressions.md) - Arrow-form switch syntax used throughout these examples
