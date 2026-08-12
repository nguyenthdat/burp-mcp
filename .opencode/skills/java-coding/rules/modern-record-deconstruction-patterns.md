# modern-record-deconstruction-patterns

> Use record patterns to deconstruct records in `switch`/`instanceof`

## Why It Matters

Without record patterns (finalized in Java 21), extracting a record's components inside a conditional requires an explicit cast followed by manual accessor calls, repeated at every nesting level for composed records. Record patterns let you deconstruct a record directly in the pattern itself, binding each component to a local variable in one step, including nested records, which removes a whole layer of accessor-chasing boilerplate.

## Bad

```java
record Point(int x, int y) {}
record Line(Point start, Point end) {}

double length(Object shape) {
    if (shape instanceof Line) {
        Line line = (Line) shape;
        Point start = line.start(); // manual accessor calls
        Point end = line.end();
        int dx = end.x() - start.x();
        int dy = end.y() - start.y();
        return Math.sqrt(dx * dx + dy * dy);
    }
    return 0;
}
```

## Good

```java
record Point(int x, int y) {}
record Line(Point start, Point end) {}

double length(Object shape) {
    if (shape instanceof Line(Point(int x1, int y1), Point(int x2, int y2))) {
        int dx = x2 - x1;
        int dy = y2 - y1;
        return Math.sqrt(dx * dx + dy * dy);
    }
    return 0;
}
```

## Record Patterns in `switch`

Deconstruction reads especially well combined with sealed hierarchies:

```java
sealed interface Shape permits Circle, Line {}
record Circle(Point center, double radius) implements Shape {}

String describe(Shape shape) {
    return switch (shape) {
        case Circle(Point(var cx, var cy), var r) ->
                "Circle at (%d,%d) r=%.1f".formatted(cx, cy, r);
        case Line(Point(var x1, var y1), Point(var x2, var y2)) ->
                "Line (%d,%d)->(%d,%d)".formatted(x1, y1, x2, y2);
    };
}
```

## Mixing `var` and Explicit Types

Component patterns can use `var` to infer the component's declared type, or spell it out explicitly - both are legal, but mixing styles inconsistently within one pattern hurts readability. Prefer `var` when the record's own declaration already documents the types clearly.

## See Also

- [`modern-records-immutable-data`](modern-records-immutable-data.md) - Records are the data shape these patterns deconstruct
- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - Exhaustive switch pairs naturally with record patterns
- [`modern-guarded-patterns-when`](modern-guarded-patterns-when.md) - Adding a `when` condition after a deconstruction pattern
- [`type-pattern-matching-instanceof`](type-pattern-matching-instanceof.md) - The simpler, non-deconstructing form of pattern matching
