# anti-god-class

> Don't build a God class with too many responsibilities

## Why It Matters

A class that knows how to validate orders, calculate pricing, send emails, and write to the database becomes the one file every developer is afraid to touch, because any change risks breaking three unrelated features at once. God classes also make unit testing nearly impossible, since exercising one responsibility drags in the setup for all the others.

## Bad

```java
public class OrderManager {
  // Validation
  public boolean validateOrder(Order order) { /* ... */ return true; }

  // Pricing
  public BigDecimal calculateTotal(Order order) { /* ... */ return BigDecimal.ZERO; }
  public BigDecimal applyDiscount(BigDecimal total, Customer c) { /* ... */ return total; }

  // Persistence
  public void saveOrder(Order order) { /* JDBC calls directly in here */ }

  // Notifications
  public void sendConfirmationEmail(Order order) { /* SMTP calls directly in here */ }
  public void sendSmsAlert(Order order) { /* Twilio calls directly in here */ }

  // Reporting
  public String generateInvoicePdf(Order order) { /* PDF generation in here */ return ""; }

  // 1,800 lines, 40 methods, touched by every team, understood by no one
}
```

## Good

```java
public class OrderValidator {
  public ValidationResult validate(Order order) { /* ... */ return ValidationResult.ok(); }
}

public class PricingService {
  public BigDecimal calculateTotal(Order order, Customer customer) { /* ... */ return BigDecimal.ZERO; }
}

public class OrderRepository {
  public void save(Order order) { /* ... */ }
}

public class OrderNotifier {
  public void notifyConfirmation(Order order) { /* ... */ }
}

// A thin coordinator wires the focused collaborators together
public class OrderProcessor {
  private final OrderValidator validator;
  private final PricingService pricing;
  private final OrderRepository repository;
  private final OrderNotifier notifier;

  public OrderProcessor(OrderValidator validator, PricingService pricing,
      OrderRepository repository, OrderNotifier notifier) {
    this.validator = validator;
    this.pricing = pricing;
    this.repository = repository;
    this.notifier = notifier;
  }

  public void process(Order order, Customer customer) {
    validator.validate(order).throwIfInvalid();
    order.setTotal(pricing.calculateTotal(order, customer));
    repository.save(order);
    notifier.notifyConfirmation(order);
  }
}
```

## A Practical Warning Sign

```java
// PMD's design.xml/CyclomaticComplexity and ExcessiveClassLength rules exist
// specifically to flag this before it gets out of hand:
// - more than ~15 public methods
// - more than ~500-1000 lines
// - a name ending in "Manager", "Processor", "Utils", or "Helper" that
//   keeps growing because "it's already the place things go"
```

## See Also

- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keeping each class's public API small and focused
- [`api-composition-over-inheritance`](api-composition-over-inheritance.md) - Composing focused collaborators instead of one large class
- [`anti-overuse-of-static`](anti-overuse-of-static.md) - Static-utility God classes are a common variant of this same problem
- [`lint-pmd-rulesets`](lint-pmd-rulesets.md) - Static analysis rules that flag excessive class size/complexity
