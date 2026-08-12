# err-exception-message-context

> Include actionable context in exception messages

## Why It Matters

An exception message like "invalid input" or "not found" tells whoever reads the log absolutely nothing about which input, which record, or what would fix it — they have to go spelunking through the stack trace and surrounding code to reconstruct what the message should have told them directly. A good message includes the offending value, the expected constraint, and enough identifying detail (IDs, keys, paths) to act on the failure without re-running the program in a debugger.

## Bad

```java
public void validateAge(int age) {
    if (age < 0 || age > 150) {
        throw new IllegalArgumentException("invalid age"); // invalid how? what was it?
    }
}

public User findUser(String id) {
    User user = repository.get(id);
    if (user == null) {
        throw new NoSuchElementException("not found"); // not found... what? where?
    }
    return user;
}
```

## Good

```java
public void validateAge(int age) {
    if (age < 0 || age > 150) {
        throw new IllegalArgumentException(
                "age must be between 0 and 150, got: " + age);
    }
}

public User findUser(String id) {
    User user = repository.get(id);
    if (user == null) {
        throw new NoSuchElementException("no user found with id '" + id + "'");
    }
    return user;
}
```

## Structured Context For Complex Failures

```java
public class OrderValidationException extends RuntimeException {

    public OrderValidationException(String orderId, String field, Object value, String constraint) {
        super("order %s failed validation: field '%s' with value '%s' violates constraint: %s"
                .formatted(orderId, field, value, constraint));
    }
}

throw new OrderValidationException("ord-4471", "quantity", -3, "must be positive");
// "order ord-4471 failed validation: field 'quantity' with value '-3' violates constraint: must be positive"
```

## Avoid Leaking Sensitive Data

Context should be actionable for operators, not a dump of secrets. Include IDs and constraint descriptions; avoid embedding raw passwords, tokens, or full PII in exception messages that may end up in shared logs.

## See Also

- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Validate arguments early and fail fast
- [`null-optional-orElseThrow`](null-optional-orElseThrow.md) - Use `orElseThrow` with a meaningful exception supplier
- [`err-exception-chaining`](err-exception-chaining.md) - Chain causes via constructor
- [`modern-text-blocks`](modern-text-blocks.md) - Use text blocks for multi-line messages
