# null-optional-chaining

> Chain `Optional` with `map`/`flatMap`/`filter` instead of manual null checks

## Why It Matters

Manually unwrapping an `Optional` with `isPresent()`/`get()` and then null-checking the next step in the chain reintroduces the exact nested-if pyramid `Optional` was meant to eliminate. The functional pipeline methods let you describe the transformation once and let `Optional` propagate absence automatically, producing flatter, more declarative code.

## Bad

```java
public String resolveCity(Optional<Address> maybeAddress) {
    if (maybeAddress.isPresent()) {
        Address address = maybeAddress.get();
        if (address.city() != null) {
            String city = address.city();
            if (!city.isBlank()) {
                return city.toUpperCase();
            }
        }
    }
    return "UNKNOWN";
}
```

## Good

```java
public String resolveCity(Optional<Address> maybeAddress) {
    return maybeAddress
            .map(Address::city)
            .filter(city -> !city.isBlank())
            .map(String::toUpperCase)
            .orElse("UNKNOWN");
}
```

## flatMap For Nested Optionals

```java
public record Address(String street, Optional<String> unit) {}

public Optional<String> unitLabel(Optional<Address> maybeAddress) {
    // map would produce Optional<Optional<String>> - flatMap flattens it
    return maybeAddress.flatMap(Address::unit);
}
```

## Combining Two Optionals

```java
public Optional<FullOrder> combine(Optional<Customer> customer, Optional<Cart> cart) {
    return customer.flatMap(c -> cart.map(cart2 -> new FullOrder(c, cart2)));
}
```

## See Also

- [`null-avoid-isPresent-get`](null-avoid-isPresent-get.md) - Avoid `isPresent()` + `get()`; use functional methods
- [`null-optional-orElseThrow`](null-optional-orElseThrow.md) - Use `orElseThrow` with a meaningful exception supplier
- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Use streams for data transformation
