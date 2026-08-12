# api-tostring-diagnostics

> Override `toString()` for diagnostics

## Why It Matters

The default `Object.toString()` prints the class name and a hash code, which tells a developer staring at a log line or a failed assertion absolutely nothing about what went wrong. A well-written `toString()` turns every log statement, exception message, and debugger inspection into useful information instead of `com.example.Order@4554617c`, cutting incident and debugging time significantly.

## Bad

```java
public class Order {
    private final String id;
    private final BigDecimal total;
    private final OrderStatus status;

    public Order(String id, BigDecimal total, OrderStatus status) {
        this.id = id;
        this.total = total;
        this.status = status;
    }
    // no toString() override
}

Order order = new Order("ORD-42", new BigDecimal("19.99"), OrderStatus.PENDING);
log.warn("Failed to process order: {}", order);
// Logs: Failed to process order: com.example.Order@4554617c
// Completely useless for diagnosing the issue
```

## Good

```java
public class Order {
    private final String id;
    private final BigDecimal total;
    private final OrderStatus status;

    public Order(String id, BigDecimal total, OrderStatus status) {
        this.id = id;
        this.total = total;
        this.status = status;
    }

    @Override
    public String toString() {
        return "Order{id='%s', total=%s, status=%s}".formatted(id, total, status);
    }
}

Order order = new Order("ORD-42", new BigDecimal("19.99"), OrderStatus.PENDING);
log.warn("Failed to process order: {}", order);
// Logs: Failed to process order: Order{id='ORD-42', total=19.99, status=PENDING}
```

## Records Provide This Automatically

```java
public record Order(String id, BigDecimal total, OrderStatus status) {}

Order order = new Order("ORD-42", new BigDecimal("19.99"), OrderStatus.PENDING);
System.out.println(order);
// Order[id=ORD-42, total=19.99, status=PENDING] - generated, no code needed
```

## Excluding Sensitive Data

```java
public final class UserSession {
    private final String userId;
    private final String authToken; // must never appear in logs

    public UserSession(String userId, String authToken) {
        this.userId = userId;
        this.authToken = authToken;
    }

    @Override
    public String toString() {
        // Deliberately omit authToken - toString output often ends up in logs
        return "UserSession{userId='%s', authToken='[REDACTED]'}".formatted(userId);
    }
}
```

## See Also

- [`api-equals-hashcode-contract`](api-equals-hashcode-contract.md) - The other pair of methods typically overridden together with toString
- [`api-record-data-carrier`](api-record-data-carrier.md) - Records generate a readable toString for free
- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Documenting the format toString produces if callers may parse it
- [`err-exception-message-context`](err-exception-message-context.md) - Using toString output to enrich exception messages
