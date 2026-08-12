# anti-excessive-checked-exceptions

> Don't force callers through excessive checked exceptions

## Why It Matters

When every method in a call chain throws a different checked exception, callers either write a wall of near-identical `catch` blocks or - far more often - swallow everything with `catch (Exception e) {}` just to get the code to compile. Checked exceptions are meant for recoverable conditions the caller can meaningfully act on; overusing them trains developers to treat exception handling as a compiler-appeasement chore instead of a design decision.

## Bad

```java
public interface PaymentGateway {
  // Five checked exceptions for one operation - callers can't reasonably
  // handle each differently, so they either duplicate boilerplate or swallow all of it
  void charge(Order order)
      throws NetworkTimeoutException, InvalidCardException,
             GatewayMaintenanceException, CurrencyMismatchException,
             FraudSuspicionException;
}

// Typical caller response: give up and catch everything
try {
  gateway.charge(order);
} catch (Exception e) { // Swallows real bugs along with expected failures
  log.error("payment failed", e);
}
```

## Good

```java
// Group genuinely-recoverable conditions behind one exception hierarchy,
// and let truly unexpected conditions be unchecked.
public class PaymentException extends Exception {
  public enum Reason { NETWORK, INVALID_CARD, FRAUD_SUSPECTED, CURRENCY_MISMATCH }

  private final Reason reason;

  public PaymentException(Reason reason, String message, Throwable cause) {
    super(message, cause);
    this.reason = reason;
  }

  public Reason reason() {
    return reason;
  }
}

public interface PaymentGateway {
  void charge(Order order) throws PaymentException;
}

// Caller handles the one exception type, branching only where it actually matters
try {
  gateway.charge(order);
} catch (PaymentException e) {
  switch (e.reason()) {
    case FRAUD_SUSPECTED -> flagForReview(order);
    case NETWORK -> retryQueue.enqueue(order);
    default -> notifyCustomerOfFailure(order, e);
  }
}
```

## When Multiple Checked Exceptions Are Still Right

```java
// If callers genuinely need to react differently and recovery is common,
// a small, well-documented set of distinct exceptions is fine:
public interface FileImporter {
  void importFile(Path path) throws FileNotFoundException, InvalidFormatException;
  // Two exceptions, two genuinely different and common recovery paths - OK.
}
```

## See Also

- [`err-checked-vs-unchecked`](err-checked-vs-unchecked.md) - The positive rule for choosing between checked and unchecked
- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - How to design one exception type that carries enough context
- [`anti-catch-and-ignore`](anti-catch-and-ignore.md) - The swallow-everything pattern this anti-pattern encourages
