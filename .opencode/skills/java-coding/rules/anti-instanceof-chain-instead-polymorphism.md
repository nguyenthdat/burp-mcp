# anti-instanceof-chain-instead-polymorphism

> Don't chain `instanceof` where polymorphism/pattern matching fits

## Why It Matters

A long `instanceof` chain duplicates the type-dispatch logic the object system already gives you for free, and every time a new subtype is added, someone has to remember to update every single chain scattered across the codebase - miss one, and it silently falls through to a default case. Polymorphism (or, for closed hierarchies, an exhaustive sealed-type switch) puts that dispatch in one place the compiler can verify is complete.

## Bad

```java
public double calculateArea(Shape shape) {
  if (shape instanceof Circle) {
    Circle c = (Circle) shape;
    return Math.PI * c.getRadius() * c.getRadius();
  } else if (shape instanceof Rectangle) {
    Rectangle r = (Rectangle) shape;
    return r.getWidth() * r.getHeight();
  } else if (shape instanceof Triangle) {
    Triangle t = (Triangle) shape;
    return 0.5 * t.getBase() * t.getHeight();
  } else {
    throw new IllegalArgumentException("unknown shape: " + shape);
    // Adding a new Shape subtype means hunting down every such chain
    // in the codebase and remembering to update each one
  }
}
```

## Good

```java
// Option 1: polymorphism - each subtype owns its own behavior
public abstract sealed class Shape permits Circle, Rectangle, Triangle {
  public abstract double area();
}

public final class Circle extends Shape {
  private final double radius;
  public Circle(double radius) { this.radius = radius; }
  @Override public double area() { return Math.PI * radius * radius; }
}

// Calling code no longer needs to know about subtypes at all
double area = shape.area();
```

```java
// Option 2: exhaustive sealed-type pattern matching (Java 21+),
// when the logic doesn't belong on the type itself
public double calculateArea(Shape shape) {
  return switch (shape) {
    case Circle c -> Math.PI * c.radius() * c.radius();
    case Rectangle r -> r.width() * r.height();
    case Triangle t -> 0.5 * t.base() * t.height();
    // No `default` needed - compiler verifies exhaustiveness over sealed permits
  };
}
```

## When an instanceof Chain Is Still Reasonable

```java
// A one-off check against an unsealed, third-party type hierarchy you
// don't control, with only two cases, is fine as plain instanceof:
if (exception instanceof SQLException sqlEx) {
  handleSqlError(sqlEx);
} else {
  handleGenericError(exception);
}
```

## See Also

- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - The positive rule for exhaustive, compiler-checked dispatch
- [`api-sealed-closed-hierarchy`](api-sealed-closed-hierarchy.md) - Sealing the hierarchy so the compiler can verify exhaustiveness
- [`api-composition-over-inheritance`](api-composition-over-inheritance.md) - When polymorphism itself is the better fit over any type-switch
- [`type-pattern-matching-instanceof`](type-pattern-matching-instanceof.md) - Modern instanceof pattern binding syntax
