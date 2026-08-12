# proj-package-private-default

> Default new types/members to package-private

## Why It Matters

`public` is a one-way door: once a type or method ships as part of the public API, removing or changing it breaks every caller and typically requires a deprecation cycle. Defaulting to package-private (no modifier) keeps implementation details invisible outside the package, letting you refactor freely as long as the package's genuinely public surface stays stable. Widening visibility later is trivial; narrowing it after external code depends on it is not.

## Bad

```java
package com.example.app.orders;

public class OrderValidator {          // Public, but only ever used by OrderService in this package
    public boolean isValid(Order order) {
        return checkInventory(order) && checkPayment(order);
    }

    public boolean checkInventory(Order order) { ... }  // Implementation detail, exposed anyway
    public boolean checkPayment(Order order) { ... }    // Implementation detail, exposed anyway
}

public class OrderService {
    private final OrderValidator validator = new OrderValidator();
    // ...
}
```

## Good

```java
package com.example.app.orders;

class OrderValidator {                 // Package-private: only visible within com.example.app.orders
    boolean isValid(Order order) {
        return checkInventory(order) && checkPayment(order);
    }

    private boolean checkInventory(Order order) { ... }  // Private: only used within this class
    private boolean checkPayment(Order order) { ... }
}

public class OrderService {            // Public: the intended entry point for this feature
    private final OrderValidator validator = new OrderValidator();

    public OrderResult submit(Order order) {
        if (!validator.isValid(order)) {
            throw new InvalidOrderException(order.id());
        }
        // ...
        return OrderResult.accepted(order.id());
    }
}
```

## Widening Visibility Later

```java
// If OrderValidator later needs to be reused by a sibling package, widen it
// deliberately at that point - the compiler will catch every place that
// needs adjustment, and the change is a conscious API decision, not a default.
public class OrderValidator {
    public boolean isValid(Order order) { ... }
}
```

## See Also

- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by technical layer
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API small and intentional
- [`api-final-classes-not-designed-for-inheritance`](api-final-classes-not-designed-for-inheritance.md) - Seal classes not designed for extension
