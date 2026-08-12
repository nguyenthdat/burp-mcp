# modern-records-immutable-data

> Use records instead of hand-written immutable data classes

## Why It Matters

Before records (Java 16+), an immutable data carrier required manually writing a constructor, accessors, `equals`, `hashCode`, and `toString` - dozens of lines of boilerplate that are easy to get subtly wrong (forgetting a field in `equals`, mismatching accessor naming) and tedious to keep in sync as fields change. Records generate all of this correctly from a single declaration, so the class definition communicates intent instead of ceremony.

## Bad

```java
public final class Point {
    private final int x;
    private final int y;

    public Point(int x, int y) {
        this.x = x;
        this.y = y;
    }

    public int getX() { return x; }
    public int getY() { return y; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof Point)) return false;
        Point point = (Point) o;
        return x == point.x && y == point.y; // easy to forget a field here
    }

    @Override
    public int hashCode() {
        return Objects.hash(x, y);
    }

    @Override
    public String toString() {
        return "Point{x=" + x + ", y=" + y + "}";
    }
}
```

## Good

```java
public record Point(int x, int y) {}

// equals(), hashCode(), toString(), x(), and y() are all generated correctly
Point p1 = new Point(1, 2);
Point p2 = new Point(1, 2);
System.out.println(p1.equals(p2)); // true
System.out.println(p1);            // Point[x=1, y=2]
```

## Records Can Still Have Behavior

```java
public record Point(int x, int y) {
    // Compact constructor for validation
    public Point {
        if (x < 0 || y < 0) {
            throw new IllegalArgumentException("Coordinates must be non-negative");
        }
    }

    // Derived methods are welcome - records aren't limited to plain data
    public double distanceTo(Point other) {
        int dx = x - other.x;
        int dy = y - other.y;
        return Math.sqrt(dx * dx + dy * dy);
    }

    // Additional static factories are common too
    public static Point origin() {
        return new Point(0, 0);
    }
}
```

## When a Record Is Not Appropriate

Records are unsuitable when the type needs mutable state, needs to participate in a class-based inheritance hierarchy (records are implicitly `final` and cannot extend another class), or when its "meaning" is identity-based rather than value-based (e.g. a JPA entity typically should not be a record).

## See Also

- [`api-record-data-carrier`](api-record-data-carrier.md) - API design guidance for choosing records as your data carrier
- [`api-record-compact-constructor-validation`](api-record-compact-constructor-validation.md) - Validating invariants in the compact constructor
- [`modern-record-deconstruction-patterns`](modern-record-deconstruction-patterns.md) - Destructuring records in `switch` and `instanceof`
- [`api-equals-hashcode-contract`](api-equals-hashcode-contract.md) - What records give you for free versus what to check manually
