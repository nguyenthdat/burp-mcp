# modern-sealed-interfaces-hierarchy

> Use sealed interfaces to model closed hierarchies

## Why It Matters

A plain interface allows any class, anywhere, to implement it - which is exactly right for extensibility points like `Comparable`, but wrong for domain types where the full set of variants is known and fixed (a payment method is a card, a bank transfer, or cash - never anything else). Sealed interfaces (Java 17+) let the compiler enforce that closed set, which in turn enables exhaustive `switch` pattern matching with no `default` branch required.

## Bad

```java
// Open interface - any class in any package could implement this
public interface PaymentMethod {
    String describe();
}

public class CardPayment implements PaymentMethod {
    public String describe() { return "Card"; }
}

public class BankTransfer implements PaymentMethod {
    public String describe() { return "Bank Transfer"; }
}

// Nothing stops an unrelated, unexpected implementation from appearing
public class CryptoPayment implements PaymentMethod {
    public String describe() { return "Crypto"; }
}

double fee(PaymentMethod method) {
    if (method instanceof CardPayment) return 0.03;
    if (method instanceof BankTransfer) return 0.01;
    // CryptoPayment silently falls through with no compile-time warning
    throw new IllegalStateException("Unknown payment method");
}
```

## Good

```java
public sealed interface PaymentMethod permits CardPayment, BankTransfer, CashPayment {
    String describe();
}

public record CardPayment(String last4) implements PaymentMethod {
    public String describe() { return "Card ending in " + last4; }
}

public record BankTransfer(String iban) implements PaymentMethod {
    public String describe() { return "Bank transfer"; }
}

public record CashPayment() implements PaymentMethod {
    public String describe() { return "Cash"; }
}

double fee(PaymentMethod method) {
    return switch (method) {
        case CardPayment c -> 0.03;
        case BankTransfer b -> 0.01;
        case CashPayment c -> 0.0;
        // no default - compiler guarantees every permitted type is handled
    };
}
```

## Permitting Subtypes Across Files

Permitted subtypes can live in the same file as the sealed interface, or in separate files within the same module/package, as long as each explicitly declares `final`, `sealed`, or `non-sealed`:

```java
public sealed interface Shape permits Circle, Polygon {}

public final class Circle implements Shape { /* ... */ }

// non-sealed reopens this one branch to further extension, if truly needed
public non-sealed class Polygon implements Shape { /* ... */ }
```

Prefer `final` (or a `record`, which is implicitly final) for permitted subtypes unless you have a specific reason to reopen the hierarchy with `non-sealed` - doing so re-introduces the exhaustiveness gap this rule exists to close.

## See Also

- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - Exhaustive switch over sealed hierarchies
- [`api-sealed-closed-hierarchy`](api-sealed-closed-hierarchy.md) - Broader API design guidance for sealed types
- [`modern-records-immutable-data`](modern-records-immutable-data.md) - Records are the natural implementation for sealed permitted types
- [`modern-guarded-patterns-when`](modern-guarded-patterns-when.md) - Refining sealed-type switch cases with `when` guards
