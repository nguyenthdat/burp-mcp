# err-checked-vs-unchecked

> Choose checked vs unchecked exceptions by recoverability

## Why It Matters

Checked exceptions should signal conditions a competent caller can reasonably recover from (a missing file, a network timeout); unchecked exceptions should signal programming errors or unrecoverable states (a null argument, a violated invariant) that no amount of `catch` blocks will fix. Getting this backwards either forces callers to write meaningless `catch` blocks around programmer errors, or lets genuinely recoverable failures crash the application silently as unchecked exceptions nobody was told to expect.

## Bad

```java
// Unchecked exception for a genuinely recoverable condition - caller has no compiler
// reminder that this can fail, and easily lets it crash the whole request handler.
public class PaymentGateway {

    public Receipt charge(Card card, BigDecimal amount) {
        if (!network.isReachable()) {
            throw new RuntimeException("network unavailable"); // silently forgettable
        }
        return network.send(card, amount);
    }
}

// Checked exception for a pure programming error - forces every caller
// to add a useless try/catch for something that should never happen.
public class ConfigLoader {

    public String get(String key) throws MissingKeyException {
        if (key == null) {
            throw new MissingKeyException("key is null"); // this is a bug, not a recoverable state
        }
        return values.get(key);
    }
}
```

## Good

```java
// Checked: caller can retry, queue for later, or show a "try again" message
public class PaymentGateway {

    public Receipt charge(Card card, BigDecimal amount) throws PaymentUnavailableException {
        if (!network.isReachable()) {
            throw new PaymentUnavailableException("payment network unreachable");
        }
        return network.send(card, amount);
    }
}

// Unchecked: a null key is a programmer error, not something to recover from at runtime
public class ConfigLoader {

    public String get(String key) {
        Objects.requireNonNull(key, "key must not be null");
        String value = values.get(key);
        if (value == null) {
            throw new NoSuchElementException("no config value for key: " + key);
        }
        return value;
    }
}
```

## Rule Of Thumb

- Checked: the caller has a realistic, distinct recovery action (retry, fallback, ask the user).
- Unchecked: precondition violations, invariant breaks, or "this should be impossible" states.
- When in doubt, prefer unchecked — checked exceptions that get wrapped and rethrown everywhere add ceremony without adding safety (`err-unchecked-wrap-checked`).

## See Also

- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - Build a custom exception hierarchy for domain errors
- [`err-unchecked-wrap-checked`](err-unchecked-wrap-checked.md) - Wrap checked exceptions instead of propagating `throws` everywhere
- [`anti-excessive-checked-exceptions`](anti-excessive-checked-exceptions.md) - Avoid excessive checked exceptions
- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Validate arguments early and fail fast
