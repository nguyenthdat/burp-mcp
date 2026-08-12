# api-sealed-closed-hierarchy

> Use sealed classes/interfaces for closed hierarchies

## Why It Matters

When a hierarchy models a fixed, known set of variants — a payment method, a shape, a result type — leaving it open for arbitrary subclassing means the compiler can never guarantee a `switch` handles every case, and any code depending on exhaustiveness silently rots as new subtypes appear elsewhere. Sealing the hierarchy tells both the compiler and future readers exactly which types exist, enabling exhaustive `switch` expressions that fail to compile the moment a new variant is added without being handled.

## Bad

```java
public interface Shape {
    double area();
}

public class Circle implements Shape {
    private final double radius;
    public Circle(double radius) { this.radius = radius; }
    public double area() { return Math.PI * radius * radius; }
}

public class Rectangle implements Shape {
    private final double width, height;
    public Rectangle(double width, double height) { this.width = width; this.height = height; }
    public double area() { return width * height; }
}

// Anyone, anywhere, can add a new implementation
public class Triangle implements Shape { /* ... */ public double area() { return 0; } }

// Callers can never write an exhaustive switch with confidence
static String describe(Shape shape) {
    if (shape instanceof Circle c) {
        return "circle";
    } else if (shape instanceof Rectangle r) {
        return "rectangle";
    }
    return "unknown"; // silently swallows new shapes forever
}
```

## Good

```java
public sealed interface Shape permits Circle, Rectangle, Triangle {
    double area();
}

public record Circle(double radius) implements Shape {
    public double area() { return Math.PI * radius * radius; }
}

public record Rectangle(double width, double height) implements Shape {
    public double area() { return width * height; }
}

public record Triangle(double base, double height) implements Shape {
    public double area() { return 0.5 * base * height; }
}

// Compiler enforces exhaustiveness - adding a new permitted type breaks this build
static String describe(Shape shape) {
    return switch (shape) {
        case Circle c -> "circle";
        case Rectangle r -> "rectangle";
        case Triangle t -> "triangle";
        // no default needed - the compiler knows these are all the cases
    };
}
```

## Sealing With Non-Record Subtypes

```java
public sealed abstract class Command permits StartCommand, StopCommand {
    public abstract void execute();
}

public final class StartCommand extends Command {
    public void execute() { /* ... */ }
}

public final class StopCommand extends Command {
    public void execute() { /* ... */ }
}
```

Subtypes must be `final`, `sealed`, or `non-sealed`, and must reside in the same module (or package, for unnamed modules) as the sealed supertype — this is what makes the closed set enforceable.

## See Also

- [`modern-sealed-interfaces-hierarchy`](modern-sealed-interfaces-hierarchy.md) - Broader patterns for sealed hierarchies
- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - Exhaustive switches over sealed types
- [`api-final-classes-not-designed-for-inheritance`](api-final-classes-not-designed-for-inheritance.md) - Related discipline for non-sealed hierarchies
- [`modern-guarded-patterns-when`](modern-guarded-patterns-when.md) - Adding conditions to pattern branches
