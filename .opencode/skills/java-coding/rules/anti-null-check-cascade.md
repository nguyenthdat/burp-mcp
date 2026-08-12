# anti-null-check-cascade

> Don't cascade repetitive null checks instead of `Optional`/`Objects`

## Why It Matters

Nested `if (x != null)` chains hide the actual business logic inside a pyramid of defensive code, and every additional field in the chain doubles the number of paths a reviewer has to mentally trace. `Optional` chaining (`map`/`flatMap`/`orElse`) or `Objects.requireNonNullElse` express the same "walk down the chain, stop at the first absence" logic in a single, linear, testable expression.

## Bad

```java
public String getCityForOrder(Order order) {
  if (order != null) {
    if (order.getCustomer() != null) {
      if (order.getCustomer().getAddress() != null) {
        if (order.getCustomer().getAddress().getCity() != null) {
          return order.getCustomer().getAddress().getCity();
        }
      }
    }
  }
  return "Unknown";
  // Four nesting levels just to express "get the city, or a default"
}
```

## Good

```java
public String getCityForOrder(Order order) {
  return Optional.ofNullable(order)
      .map(Order::getCustomer)
      .map(Customer::getAddress)
      .map(Address::getCity)
      .orElse("Unknown");
}
```

## For a Single Value, Objects Utility Methods Are Even Lighter

```java
public String describe(String nickname, String fallback) {
  // Instead of: if (nickname != null) { return nickname; } else { return fallback; }
  return Objects.requireNonNullElse(nickname, fallback);
}

public void validate(String username) {
  // Instead of: if (username == null) { throw new NullPointerException(...); }
  Objects.requireNonNull(username, "username must not be null");
}
```

## The Real Fix Is Often Upstream

```java
// If Order/Customer/Address are your own domain types, consider making
// the fields non-nullable and enforcing that at construction time instead
// of pushing the null-checking burden onto every reader.
public record Address(String city, String zipCode) {
  public Address {
    Objects.requireNonNull(city, "city must not be null");
  }
}
```

## See Also

- [`null-optional-chaining`](null-optional-chaining.md) - The positive rule for expressing this logic idiomatically
- [`null-requireNonNull-guard`](null-requireNonNull-guard.md) - Using `Objects.requireNonNull` at construction to prevent the cascade from ever being needed
- [`anti-return-null-instead-optional`](anti-return-null-instead-optional.md) - A common root cause: APIs that return null instead of Optional
- [`null-defensive-copy`](null-defensive-copy.md) - Related discipline for keeping state consistently non-null
