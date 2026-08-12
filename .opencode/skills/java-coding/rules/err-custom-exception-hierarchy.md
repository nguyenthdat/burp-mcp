# err-custom-exception-hierarchy

> Build a custom exception hierarchy for domain errors

## Why It Matters

Throwing generic `RuntimeException` or `Exception` everywhere forces callers into catching either everything or nothing, since there's no type distinction between "the card was declined" and "the database connection dropped." A purpose-built hierarchy rooted in a single base type lets calling code catch broadly when it just needs to know "something in this domain failed" and narrowly when it needs to react differently to specific failures.

## Bad

```java
public class PaymentService {

    public void charge(Card card, BigDecimal amount) {
        if (isExpired(card)) {
            throw new RuntimeException("card expired"); // indistinguishable from any other failure
        }
        if (!hasSufficientFunds(card, amount)) {
            throw new RuntimeException("insufficient funds");
        }
        // ...
    }
}

// Caller cannot react differently to different failure modes
try {
    paymentService.charge(card, amount);
} catch (RuntimeException e) {
    // is this expired card, insufficient funds, or a bug? no way to tell
    log.error("payment failed", e);
}
```

## Good

```java
public sealed class PaymentException extends Exception
        permits CardExpiredException, InsufficientFundsException, GatewayUnavailableException {

    protected PaymentException(String message) {
        super(message);
    }

    protected PaymentException(String message, Throwable cause) {
        super(message, cause);
    }
}

public final class CardExpiredException extends PaymentException {
    public CardExpiredException(String message) {
        super(message);
    }
}

public final class InsufficientFundsException extends PaymentException {
    public InsufficientFundsException(String message) {
        super(message);
    }
}

public final class GatewayUnavailableException extends PaymentException {
    public GatewayUnavailableException(String message, Throwable cause) {
        super(message, cause);
    }
}
```

```java
try {
    paymentService.charge(card, amount);
} catch (CardExpiredException e) {
    promptForNewCard();
} catch (InsufficientFundsException e) {
    offerAlternatePaymentMethod();
} catch (GatewayUnavailableException e) {
    retryLater(e);
}
```

## Pattern Matching On Sealed Exceptions

```java
// Java 21+ pattern matching for switch works well with a sealed hierarchy
String userMessage = switch (paymentException) {
    case CardExpiredException e -> "Your card has expired. Please update it.";
    case InsufficientFundsException e -> "Insufficient funds for this transaction.";
    case GatewayUnavailableException e -> "Payment service is temporarily unavailable.";
};
```

## See Also

- [`err-checked-vs-unchecked`](err-checked-vs-unchecked.md) - Choose checked vs unchecked by recoverability
- [`err-specific-catch-order`](err-specific-catch-order.md) - Catch specific exceptions before general ones
- [`api-sealed-closed-hierarchy`](api-sealed-closed-hierarchy.md) - Use `sealed` for closed hierarchies
- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - Pattern matching for switch
