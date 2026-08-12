# anti-anemic-domain-model

> Don't reduce domain objects to anemic getter/setter bags

## Why It Matters

An anemic model - all fields, getters, and setters with zero behavior - pushes every business rule out into separate "service" classes, so the same invariant (e.g., "an order total can't go negative") ends up re-implemented, and re-forgotten, in every service that happens to touch that field. Object-oriented design puts behavior next to the data it operates on specifically to prevent that duplication and drift.

## Bad

```java
public class Order {
  private BigDecimal total;
  private OrderStatus status;
  private List<LineItem> items;

  // Pure getters/setters, no behavior, no invariants enforced
  public BigDecimal getTotal() { return total; }
  public void setTotal(BigDecimal total) { this.total = total; }
  public OrderStatus getStatus() { return status; }
  public void setStatus(OrderStatus status) { this.status = status; }
  public List<LineItem> getItems() { return items; }
  public void setItems(List<LineItem> items) { this.items = items; }
}

// Business logic scattered across services, each free to violate invariants
public class OrderService {
  public void applyDiscount(Order order, BigDecimal discount) {
    order.setTotal(order.getTotal().subtract(discount)); // Nothing stops total from going negative
  }

  public void cancel(Order order) {
    order.setStatus(OrderStatus.CANCELLED); // Nothing stops cancelling an already-shipped order
  }
}
```

## Good

```java
public class Order {
  private BigDecimal total;
  private OrderStatus status;
  private final List<LineItem> items = new ArrayList<>();

  public Order(List<LineItem> items) {
    this.items.addAll(items);
    this.total = items.stream().map(LineItem::subtotal).reduce(BigDecimal.ZERO, BigDecimal::add);
    this.status = OrderStatus.PENDING;
  }

  public void applyDiscount(BigDecimal discount) {
    BigDecimal newTotal = total.subtract(discount);
    if (newTotal.signum() < 0) {
      throw new IllegalArgumentException("discount cannot exceed order total");
    }
    total = newTotal;
  }

  public void cancel() {
    if (status == OrderStatus.SHIPPED) {
      throw new IllegalStateException("cannot cancel an order that has already shipped");
    }
    status = OrderStatus.CANCELLED;
  }

  public BigDecimal getTotal() { return total; }
  public OrderStatus getStatus() { return status; }
}

// Services now orchestrate; the Order enforces its own invariants regardless of caller
public class OrderService {
  public void applyDiscount(Order order, BigDecimal discount) {
    order.applyDiscount(discount); // Invariant is enforced exactly once, inside the class
  }
}
```

## When a Thin Data Carrier Is the Right Call

```java
// DTOs crossing a serialization boundary (JSON payloads, JPA entities used
// purely for persistence mapping) are legitimately anemic - they aren't
// domain objects, they're a wire/storage format. Keep real business rules
// in the domain layer that maps to/from them.
public record OrderResponseDto(String id, BigDecimal total, String status) { }
```

## See Also

- [`anti-mutable-public-fields`](anti-mutable-public-fields.md) - A related symptom: exposing raw state instead of protected behavior
- [`api-immutable-by-default`](api-immutable-by-default.md) - Enforcing invariants through immutability where possible
- [`api-record-data-carrier`](api-record-data-carrier.md) - When a plain data carrier (like a DTO) is actually the right shape
- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Enforcing invariants at the point of mutation, not downstream
