# type-pattern-matching-switch

> Use pattern matching for `switch` over sealed types

## Why It Matters

Pattern matching for `switch` (finalized in Java 21) combines type testing, casting, and exhaustiveness checking into a single construct. When the `switch` operates over a `sealed` hierarchy, the compiler verifies every permitted subtype is handled, so adding a new subtype later produces a compile error anywhere the hierarchy is switched on instead of a silent runtime bug in a stray `if`/`else` chain.

## Bad

```java
sealed interface Shape permits Circle, Rectangle, Triangle {}
record Circle(double radius) implements Shape {}
record Rectangle(double width, double height) implements Shape {}
record Triangle(double base, double height) implements Shape {}

public double area(Shape shape) {
    // instanceof chain: no exhaustiveness check, easy to forget a case
    if (shape instanceof Circle) {
        Circle c = (Circle) shape;
        return Math.PI * c.radius() * c.radius();
    } else if (shape instanceof Rectangle) {
        Rectangle r = (Rectangle) shape;
        return r.width() * r.height();
    }
    // Triangle silently falls through to this generic error at runtime,
    // instead of failing to compile when Triangle was added
    throw new IllegalStateException("Unhandled shape: " + shape);
}
```

## Good

```java
public double area(Shape shape) {
    return switch (shape) {
        case Circle c -> Math.PI * c.radius() * c.radius();
        case Rectangle r -> r.width() * r.height();
        case Triangle t -> 0.5 * t.base() * t.height();
        // no default needed - compiler proves all permitted subtypes are covered
    };
}
```

## Adding a New Subtype

```java
sealed interface Shape permits Circle, Rectangle, Triangle, Square {}
record Square(double side) implements Shape {}

// Compile error in area(): "the switch expression does not cover all
// possible input values" - forces every switch over Shape to be updated
```

This compile-time safety net is the main reason to prefer `sealed` + `switch` pattern matching over `instanceof` chains for closed hierarchies: the compiler, not a code reviewer, catches missed cases.

## See Also

- [`api-sealed-closed-hierarchy`](api-sealed-closed-hierarchy.md) - Designing the sealed hierarchy this pattern switches over
- [`modern-sealed-interfaces-hierarchy`](modern-sealed-interfaces-hierarchy.md) - Sealed interfaces as the modeling tool paired with this rule
- [`modern-switch-expressions`](modern-switch-expressions.md) - Arrow-form switch syntax used throughout these examples
- [`modern-guarded-patterns-when`](modern-guarded-patterns-when.md) - Adding conditions to individual pattern cases
