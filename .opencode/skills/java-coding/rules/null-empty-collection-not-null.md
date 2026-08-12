# null-empty-collection-not-null

> Return empty collections, never `null`, from collection-returning methods

## Why It Matters

A collection type already has a value that means "nothing here" — an empty one. Returning `null` instead forces every caller to add a defensive `if (list != null)` check before the natural `for` loop or stream operation they wanted to write, and forgetting that check is one of the most common sources of production `NullPointerException`s.

## Bad

```java
public class OrderRepository {

    public List<Order> findByCustomer(String customerId) {
        List<Order> results = queryDatabase(customerId);
        if (results.isEmpty()) {
            return null; // caller must remember to check
        }
        return results;
    }
}

// Caller forgets the null check
List<Order> orders = repository.findByCustomer("cust-1");
for (Order order : orders) { // NPE when there are simply no orders
    process(order);
}
```

## Good

```java
public class OrderRepository {

    public List<Order> findByCustomer(String customerId) {
        return queryDatabase(customerId); // empty list when nothing matches
    }
}

// Caller code just works, no null check needed
List<Order> orders = repository.findByCustomer("cust-1");
for (Order order : orders) {
    process(order);
}

orders.stream().forEach(this::process); // also safe on an empty list
```

## Building Empty Results Cheaply

```java
// No allocation for the empty case - these are shared, immutable singletons
List<String> empty = Collections.emptyList();
Set<String> emptySet = Set.of();
Map<String, String> emptyMap = Map.of();

public List<Order> findByCustomer(String customerId) {
    if (!customerExists(customerId)) {
        return List.of(); // explicit, cheap, never null
    }
    return queryDatabase(customerId);
}
```

## See Also

- [`null-optional-return-type`](null-optional-return-type.md) - Use `Optional<T>` for scalar return types
- [`coll-immutable-factories`](coll-immutable-factories.md) - Use `List.of`/`Set.of`/`Map.of` factories
- [`anti-return-null-instead-optional`](anti-return-null-instead-optional.md) - Returning `null` instead of `Optional`
- [`anti-null-check-cascade`](anti-null-check-cascade.md) - Avoid cascading null checks
