# null-avoid-isPresent-get

> Avoid `isPresent()` + `get()`; use functional `Optional` methods

## Why It Matters

`isPresent()` followed by `get()` is essentially a null check with extra ceremony — it reintroduces the imperative, error-prone style `Optional` was designed to replace, and nothing stops a refactor from separating the check from the `get()` call, reintroducing the exact bug `Optional` prevents. The functional API (`ifPresent`, `map`, `orElse`, `orElseGet`, `orElseThrow`) keeps the presence check and the usage atomic.

## Bad

```java
public void sendWelcomeEmail(String userId) {
    Optional<User> maybeUser = userRepository.findById(userId);
    if (maybeUser.isPresent()) {
        User user = maybeUser.get(); // easy to accidentally move this outside the if
        emailService.send(user.email(), "Welcome!");
    }
}

public String describe(Optional<Product> maybeProduct) {
    if (maybeProduct.isPresent()) {
        return maybeProduct.get().name();
    } else {
        return "no product";
    }
}
```

## Good

```java
public void sendWelcomeEmail(String userId) {
    userRepository.findById(userId)
            .ifPresent(user -> emailService.send(user.email(), "Welcome!"));
}

public String describe(Optional<Product> maybeProduct) {
    return maybeProduct.map(Product::name).orElse("no product");
}
```

## ifPresentOrElse For Both Branches

```java
public void notifyStatus(Optional<Shipment> maybeShipment) {
    maybeShipment.ifPresentOrElse(
            shipment -> logger.info("Shipment {} is {}", shipment.id(), shipment.status()),
            () -> logger.warn("No shipment found for this order")
    );
}
```

## When A Direct Check Is Still Reasonable

`isPresent()` alone (without a paired `get()`) is fine when you only need a boolean, e.g. `if (cache.get(key).isPresent()) { metrics.increment("cache.hit"); }`. The anti-pattern is specifically extracting the value with `get()` right after.

## See Also

- [`null-optional-chaining`](null-optional-chaining.md) - Chain `Optional` with functional methods
- [`null-optional-orElseThrow`](null-optional-orElseThrow.md) - Use `orElseThrow` with a meaningful exception supplier
- [`anti-null-check-cascade`](anti-null-check-cascade.md) - Avoid cascading null checks
