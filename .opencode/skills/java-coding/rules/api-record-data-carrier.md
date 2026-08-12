# api-record-data-carrier

> Use records for immutable data carriers

## Why It Matters

Before records, a simple immutable data holder required a constructor, private final fields, getters, `equals`, `hashCode`, and `toString` — dozens of lines that hide a two-field concept and invite bugs when one accessor is forgotten to be updated. Records generate all of this correctly and concisely, so the data's shape is the whole class body, and reviewers can trust the contract without re-deriving it by hand.

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
        Point other = (Point) o;
        return x == other.x && y == other.y; // easy to forget a field here
    }

    @Override
    public int hashCode() {
        return Objects.hash(x, y); // must stay in sync with equals by hand
    }

    @Override
    public String toString() {
        return "Point{x=" + x + ", y=" + y + "}";
    }
    // 30+ lines to express "a pair of ints"
}
```

## Good

```java
public record Point(int x, int y) {
    // equals, hashCode, toString, and accessors are generated and stay in sync
}

// Usage
Point origin = new Point(0, 0);
Point moved = new Point(origin.x() + 1, origin.y());
System.out.println(moved); // Point[x=1, y=0]
```

## Records With Behavior

```java
public record Money(long cents, Currency currency) {

    public Money plus(Money other) {
        requireSameCurrency(other);
        return new Money(cents + other.cents, currency);
    }

    public Money minus(Money other) {
        requireSameCurrency(other);
        return new Money(cents - other.cents, currency);
    }

    private void requireSameCurrency(Money other) {
        if (!currency.equals(other.currency)) {
            throw new IllegalArgumentException("Currency mismatch: " + currency + " vs " + other.currency);
        }
    }
}
```

## When a Record Is Not Enough

Reach for a regular class when identity matters more than value (e.g. a mutable `Session` tracked by reference), when the type needs more than one internal representation, or when the class must extend another class — records can only implement interfaces.

## See Also

- [`api-record-compact-constructor-validation`](api-record-compact-constructor-validation.md) - Validate invariants when constructing a record
- [`modern-records-immutable-data`](modern-records-immutable-data.md) - Broader guidance on modeling data with records
- [`api-immutable-by-default`](api-immutable-by-default.md) - Why immutability should be the default design choice
- [`api-equals-hashcode-contract`](api-equals-hashcode-contract.md) - The contract records implement for you automatically
