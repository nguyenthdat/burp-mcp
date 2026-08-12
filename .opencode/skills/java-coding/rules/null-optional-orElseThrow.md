# null-optional-orElseThrow

> Use `orElseThrow` with a meaningful exception supplier

## Why It Matters

Calling `.get()` on an empty `Optional` throws a bare `NoSuchElementException` with a message that tells the reader nothing about what was missing or why. `orElseThrow(Supplier<X>)` lets you raise a domain-specific, actionable exception right at the point where the absence is detected, which is exactly where the most context is available.

## Bad

```java
public User getUser(String id) {
    Optional<User> maybeUser = repository.findById(id);
    return maybeUser.get(); // NoSuchElementException: No value present - no context at all
}
```

## Good

```java
public User getUser(String id) {
    return repository.findById(id)
            .orElseThrow(() -> new UserNotFoundException("No user found with id " + id));
}

public class UserNotFoundException extends RuntimeException {
    public UserNotFoundException(String message) {
        super(message);
    }
}
```

## Lazy Evaluation Matters

```java
// Bad: orElseThrow(new ...) still constructs the exception even when the value IS present,
// wasting a stack-trace capture on the happy path
User user = repository.findById(id).orElseThrow(new UserNotFoundException("id=" + id));

// Good: the supplier lambda only runs when the Optional is actually empty
User user = repository.findById(id)
        .orElseThrow(() -> new UserNotFoundException("id=" + id));
```

## Default No-Arg Form

```java
// orElseThrow() with no arguments throws NoSuchElementException -
// fine for quick prototypes, but prefer the supplier form for anything user-facing
Config config = loadConfig().orElseThrow();
```

## See Also

- [`null-optional-chaining`](null-optional-chaining.md) - Chain `Optional` with functional methods
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - Build a custom exception hierarchy for domain errors
- [`null-avoid-isPresent-get`](null-avoid-isPresent-get.md) - Avoid `isPresent()` + `get()`
