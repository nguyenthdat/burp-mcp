# err-fail-fast-validation

> Validate arguments early and fail fast

## Why It Matters

An invalid argument that isn't checked at the entry point doesn't disappear — it propagates deeper into the call stack until it causes a confusing failure far away from its actual origin, often with a stack trace that points at innocent code. Validating at the boundary turns "silent corruption three layers down" into "an immediate, precise error at the exact line where the bad input was accepted."

## Bad

```java
public class DiscountCalculator {

    public BigDecimal applyDiscount(BigDecimal price, int percentage) {
        // No validation - a negative or >100 percentage silently corrupts the result
        BigDecimal factor = BigDecimal.valueOf(100 - percentage).movePointLeft(2);
        return price.multiply(factor); // negative price emerges from a bad percentage
    }
}

// Bug surfaces far from the real cause
BigDecimal total = calculator.applyDiscount(price, 150); // -50% "discount", nobody notices
invoiceService.charge(total); // fails deep inside payment processing with a cryptic error
```

## Good

```java
public class DiscountCalculator {

    public BigDecimal applyDiscount(BigDecimal price, int percentage) {
        if (price.signum() < 0) {
            throw new IllegalArgumentException("price must not be negative: " + price);
        }
        if (percentage < 0 || percentage > 100) {
            throw new IllegalArgumentException(
                    "percentage must be between 0 and 100, got: " + percentage);
        }
        BigDecimal factor = BigDecimal.valueOf(100 - percentage).movePointLeft(2);
        return price.multiply(factor);
    }
}

// Fails immediately, at the exact call site with the bad input
calculator.applyDiscount(price, 150); // IllegalArgumentException: percentage must be between 0 and 100, got: 150
```

## Validation Belongs At Public Boundaries

```java
public record OrderRequest(String customerId, List<LineItem> items, BigDecimal total) {
    public OrderRequest {
        Objects.requireNonNull(customerId, "customerId");
        if (items.isEmpty()) {
            throw new IllegalArgumentException("order must contain at least one item");
        }
        if (total.signum() <= 0) {
            throw new IllegalArgumentException("total must be positive, got: " + total);
        }
    }
}
```

## See Also

- [`null-requireNonNull-guard`](null-requireNonNull-guard.md) - Guard constructors/methods with `Objects.requireNonNull`
- [`api-record-compact-constructor-validation`](api-record-compact-constructor-validation.md) - Validate invariants in compact constructors
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
- [`err-checked-vs-unchecked`](err-checked-vs-unchecked.md) - Choose checked vs unchecked by recoverability
