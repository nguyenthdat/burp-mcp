# anti-return-null-instead-optional

> Don't return `null` instead of `Optional`/empty collection

## Why It Matters

A `null` return forces every caller to remember to check for it, and the one caller who forgets produces a `NullPointerException` far from the actual bug's origin. Returning `Optional<T>` (for a possibly-absent single value) or an empty collection (for "no results") makes the absence explicit in the type system, so the compiler - not a stack trace at 3am - catches the missing check.

## Bad

```java
public class UserRepository {
  public User findByEmail(String email) {
    User user = queryDatabase(email);
    return user; // Returns null if not found - undocumented, easy to forget
  }

  public List<Order> findOrdersFor(String userId) {
    if (!hasOrders(userId)) {
      return null; // Caller has to guess this is possible
    }
    return loadOrders(userId);
  }
}

// Caller crashes months later when this path is finally hit
User user = repository.findByEmail("missing@example.com");
System.out.println(user.getName()); // NullPointerException
```

## Good

```java
public class UserRepository {
  public Optional<User> findByEmail(String email) {
    return Optional.ofNullable(queryDatabase(email));
  }

  public List<Order> findOrdersFor(String userId) {
    if (!hasOrders(userId)) {
      return List.of(); // Empty collection, never null
    }
    return loadOrders(userId);
  }
}

// Caller is forced by the type to handle absence
String name = repository.findByEmail("missing@example.com")
    .map(User::getName)
    .orElse("unknown");

for (Order order : repository.findOrdersFor(userId)) { // Safe even if empty
  process(order);
}
```

## Why Collections Get Different Treatment Than Single Values

```java
// Optional<List<T>> is almost always the wrong signature - it adds a second
// "emptiness" concept on top of the collection's own emptiness.
public Optional<List<Order>> findOrdersBad(String userId) { ... } // Two ways to mean "no orders"

// Just return an empty list; List.of()/Collections.emptyList() are cheap and unambiguous.
public List<Order> findOrdersGood(String userId) { ... }
```

## See Also

- [`null-optional-return-type`](null-optional-return-type.md) - The positive rule this anti-pattern violates
- [`null-empty-collection-not-null`](null-empty-collection-not-null.md) - Why collections should use empty, not null
- [`anti-null-check-cascade`](anti-null-check-cascade.md) - The defensive code this anti-pattern forces callers into
- [`null-avoid-isPresent-get`](null-avoid-isPresent-get.md) - Once you have an Optional, use it idiomatically
