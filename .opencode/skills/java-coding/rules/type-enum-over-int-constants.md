# type-enum-over-int-constants

> Use enums instead of int constants

## Why It Matters

The "int enum pattern" (`public static final int` constants) provides no type safety - any `int` can be passed where a status code is expected, constants from unrelated groups can be mixed freely, and there is no reliable way to print a meaningful name or iterate all values. Java's `enum` type solves all of these problems at compile time with no runtime overhead beyond a handful of singleton objects.

## Bad

```java
public class OrderStatus {
    public static final int PENDING = 0;
    public static final int SHIPPED = 1;
    public static final int DELIVERED = 2;
    public static final int CANCELLED = 3;
}

public class PaymentStatus {
    public static final int PENDING = 0; // same value as OrderStatus.PENDING!
    public static final int PAID = 1;
}

void updateStatus(int status) {
    if (status == OrderStatus.SHIPPED) {
        // ...
    }
}

// Compiles, but is nonsense - mixes two unrelated constant groups
updateStatus(PaymentStatus.PAID); // silently treated as OrderStatus.SHIPPED (both are 1)

// No safe way to print a name
System.out.println(OrderStatus.SHIPPED); // prints "1", not "SHIPPED"
```

## Good

```java
public enum OrderStatus {
    PENDING, SHIPPED, DELIVERED, CANCELLED
}

public enum PaymentStatus {
    PENDING, PAID
}

void updateStatus(OrderStatus status) {
    if (status == OrderStatus.SHIPPED) {
        // ...
    }
}

// updateStatus(PaymentStatus.PAID); // compile error - types are incompatible

System.out.println(OrderStatus.SHIPPED); // prints "SHIPPED"

// Enums support switch, iteration, and behavior per-constant for free
for (OrderStatus status : OrderStatus.values()) {
    System.out.println(status.ordinal() + ": " + status);
}
```

## Enums Can Carry Behavior and Data

```java
public enum OrderStatus {
    PENDING(false), SHIPPED(false), DELIVERED(true), CANCELLED(true);

    private final boolean terminal;

    OrderStatus(boolean terminal) {
        this.terminal = terminal;
    }

    public boolean isTerminal() {
        return terminal;
    }
}

if (order.status().isTerminal()) {
    archive(order);
}
```

This lets an enum encapsulate logic (`isTerminal()`) that would otherwise live in scattered `if` statements comparing raw int constants across the codebase.

## See Also

- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - Exhaustive switch pairs naturally with enums (and sealed types)
- [`api-record-data-carrier`](api-record-data-carrier.md) - Records and enums are complementary modern data-modeling tools
- [`anti-magic-numbers-strings`](anti-magic-numbers-strings.md) - The broader anti-pattern this rule corrects
