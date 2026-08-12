# proj-package-by-feature

> Organize packages by feature, not by technical layer

## Why It Matters

Packaging by technical layer (`controller`, `service`, `repository`) scatters every feature across multiple directories, so understanding or changing "orders" requires jumping between four unrelated packages, and package-private visibility becomes useless because related classes never share a package. Packaging by feature (`orders`, `customers`, `inventory`) keeps everything about one business capability together, makes package-private the natural default for internal collaborators, and lets you delete or extract a whole feature by deleting one directory.

## Bad

```
src/main/java/com/example/app/
├── controller/
│   ├── OrderController.java
│   ├── CustomerController.java
│   └── InventoryController.java
├── service/
│   ├── OrderService.java
│   ├── CustomerService.java
│   └── InventoryService.java
├── repository/
│   ├── OrderRepository.java
│   ├── CustomerRepository.java
│   └── InventoryRepository.java
└── model/
    ├── Order.java
    ├── Customer.java
    └── Inventory.java
```

```java
// To make OrderService collaborate package-privately with OrderRepository,
// everything has to be made public, since they live in different packages.
public class OrderRepository {  // Must be public - service package can't see it otherwise
    public Order findById(String id) { ... }
}
```

## Good

```
src/main/java/com/example/app/
├── orders/
│   ├── Order.java
│   ├── OrderController.java
│   ├── OrderService.java
│   └── OrderRepository.java       // Package-private where possible
├── customers/
│   ├── Customer.java
│   ├── CustomerController.java
│   ├── CustomerService.java
│   └── CustomerRepository.java
└── inventory/
    ├── Inventory.java
    ├── InventoryController.java
    ├── InventoryService.java
    └── InventoryRepository.java
```

```java
package com.example.app.orders;

class OrderRepository {  // Package-private: only OrderService in the same package needs it
    Order findById(String id) { ... }
}
```

## When Layer-Based Packages Still Make Sense

```
// A small shared-kernel package for truly cross-cutting infrastructure
// (not feature-specific) is fine alongside feature packages.
src/main/java/com/example/app/
├── orders/
├── customers/
└── common/
    ├── ApiError.java
    └── ClockProvider.java
```

## See Also

- [`proj-package-private-default`](proj-package-private-default.md) - Default new types/members to package-private
- [`proj-avoid-circular-package-deps`](proj-avoid-circular-package-deps.md) - Avoid circular package dependencies
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API small and intentional
