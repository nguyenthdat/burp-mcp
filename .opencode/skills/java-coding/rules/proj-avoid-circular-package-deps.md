# proj-avoid-circular-package-deps

> Avoid circular package dependencies

## Why It Matters

When package A depends on package B and package B depends back on package A, neither can be understood, tested, extracted, or reused in isolation - they are effectively one package artificially split in two. Circular dependencies also block modularization (JPMS forbids cyclic module dependencies entirely) and make it impossible to build or deploy one side without the other. Breaking the cycle by extracting a shared abstraction or inverting one dependency restores a clean, layered dependency graph.

## Bad

```java
package com.example.app.orders;

import com.example.app.customers.CustomerService;  // orders -> customers

public class OrderService {
    private final CustomerService customerService;
    // ...
    public void submit(Order order) {
        customerService.recordOrderHistory(order);  // Calls into customers package
    }
}
```

```java
package com.example.app.customers;

import com.example.app.orders.OrderService;  // customers -> orders : CYCLE

public class CustomerService {
    private final OrderService orderService;

    public void deactivate(String customerId) {
        orderService.cancelAllOrders(customerId);  // orders now depends on customers and vice versa
    }
}
```

## Good

```java
package com.example.app.orders;

public interface OrderEventListener {           // Abstraction owned by the "lower" package
    void onOrderSubmitted(Order order);
}

public class OrderService {
    private final List<OrderEventListener> listeners;

    public void submit(Order order) {
        // orders package has no compile-time dependency on customers at all
        listeners.forEach(listener -> listener.onOrderSubmitted(order));
    }
}
```

```java
package com.example.app.customers;

import com.example.app.orders.Order;
import com.example.app.orders.OrderEventListener;  // customers -> orders (one direction only)

public class CustomerOrderHistoryListener implements OrderEventListener {
    @Override
    public void onOrderSubmitted(Order order) {
        // Wiring happens at a higher composition layer, not inside orders itself
    }
}
```

## Detecting Cycles

```
// Use a build-time check to fail fast instead of relying on manual review:
// - ArchUnit: noClasses().that().resideInAPackage("..orders..")
//              .should().dependOnClassesThat().resideInAPackage("..customers..")
// - Gradle: the "com.autonomousapps.dependency-analysis" or module boundaries
//   plugin can flag cyclic project dependencies at build time.
```

## See Also

- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by technical layer
- [`proj-module-info-jpms`](proj-module-info-jpms.md) - Weigh JPMS `module-info.java` trade-offs deliberately
- [`api-interface-default-methods`](api-interface-default-methods.md) - Use interface default methods deliberately
- [`api-composition-over-inheritance`](api-composition-over-inheritance.md) - Favor composition over inheritance
