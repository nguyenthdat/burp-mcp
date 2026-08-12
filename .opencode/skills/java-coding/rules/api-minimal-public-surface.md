# api-minimal-public-surface

> Default to package-private; keep the public surface minimal

## Why It Matters

Every public method, field, and class is a permanent promise to callers you may never meet, and once released it must be supported forever or broken with a deprecation cycle. Defaulting to package-private access and promoting to `public` only when a genuine external caller needs it keeps the API small, makes refactoring safe, and lets tests and internal collaborators reach implementation details without leaking them to the world.

## Bad

```java
// Every member is public "just in case" - the whole class is now permanent API
public class OrderProcessor {
    public Database database;              // internal dependency, exposed
    public List<Order> pendingOrders = new ArrayList<>(); // mutable state, exposed
    public double taxRate = 0.08;           // implementation detail, exposed

    public OrderProcessor(Database database) {
        this.database = database;
    }

    public void process(Order order) {
        validate(order);
        applyTax(order);
        database.save(order);
    }

    public void validate(Order order) {     // internal step, should not be callable directly
        if (order.items().isEmpty()) {
            throw new IllegalArgumentException("Empty order");
        }
    }

    public void applyTax(Order order) {      // internal step, should not be callable directly
        order.setTotal(order.total() * (1 + taxRate));
    }
}
```

## Good

```java
public class OrderProcessor {
    private final Database database;                  // hidden dependency
    private final List<Order> pendingOrders = new ArrayList<>();
    private static final double TAX_RATE = 0.08;

    public OrderProcessor(Database database) {
        this.database = database;
    }

    // Only the entry point callers actually need is public
    public void process(Order order) {
        validate(order);
        applyTax(order);
        database.save(order);
    }

    private void validate(Order order) {
        if (order.items().isEmpty()) {
            throw new IllegalArgumentException("Empty order");
        }
    }

    private void applyTax(Order order) {
        order.setTotal(order.total() * (1 + TAX_RATE));
    }
}
```

## Package-Private for Test-Only Access

```java
class PricingEngine {
    // package-private: visible to tests in the same package,
    // invisible to other modules - no public getter needed
    BigDecimal computeDiscount(Order order) {
        return order.total().multiply(discountRate(order));
    }

    private BigDecimal discountRate(Order order) {
        return order.items().size() > 10 ? BigDecimal.valueOf(0.1) : BigDecimal.ZERO;
    }
}
```

Tests placed in the same package (e.g. `src/test/java/.../PricingEngineTest.java`) can call `computeDiscount` directly without it ever becoming part of the public API.

## See Also

- [`proj-package-by-feature`](proj-package-by-feature.md) - Structuring packages so package-private access is meaningful
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - Hiding constructors behind a narrow factory surface
- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Documenting only what is actually public
- [`anti-god-class`](anti-god-class.md) - How an ever-growing public surface signals a god class
