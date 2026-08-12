# null-requireNonNull-guard

> Guard constructors/methods with `Objects.requireNonNull`

## Why It Matters

Without an explicit guard, a `null` argument travels silently into the object's state and blows up later as an `NullPointerException` at some unrelated line, far from the actual mistake. `Objects.requireNonNull` fails fast at the boundary, with a message that names the offending parameter, turning a confusing bug hunt into an immediate, actionable stack trace.

## Bad

```java
public class OrderService {

    private final PaymentGateway gateway;
    private final NotificationService notifications;

    public OrderService(PaymentGateway gateway, NotificationService notifications) {
        this.gateway = gateway;           // no check - null accepted silently
        this.notifications = notifications;
    }

    public void submit(Order order) {
        // NPE here, far from the real bug: whoever passed a null OrderService dependency
        gateway.charge(order.total());
    }
}
```

## Good

```java
public class OrderService {

    private final PaymentGateway gateway;
    private final NotificationService notifications;

    public OrderService(PaymentGateway gateway, NotificationService notifications) {
        this.gateway = Objects.requireNonNull(gateway, "gateway must not be null");
        this.notifications = Objects.requireNonNull(notifications, "notifications must not be null");
    }

    public void submit(Order order) {
        Objects.requireNonNull(order, "order must not be null");
        gateway.charge(order.total());
        notifications.notifyOrderSubmitted(order);
    }
}
```

## Lazy Message Supplier For Expensive Messages

```java
public void process(byte[] payload) {
    // requireNonNullElseGet / message supplier avoids building the string unless needed
    Objects.requireNonNull(payload, () -> "payload for request " + currentRequestId() + " was null");
    // ...
}
```

## Compact Constructors On Records

```java
public record Money(BigDecimal amount, Currency currency) {
    public Money {
        Objects.requireNonNull(amount, "amount");
        Objects.requireNonNull(currency, "currency");
    }
}
```

## See Also

- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Validate arguments early and fail fast
- [`api-record-compact-constructor-validation`](api-record-compact-constructor-validation.md) - Validate invariants in compact constructors
- [`null-nullable-annotation`](null-nullable-annotation.md) - Annotate nullability so intent is documented
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
