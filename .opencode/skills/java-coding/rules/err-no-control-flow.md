# err-no-control-flow

> Don't use exceptions for ordinary control flow

## Why It Matters

Constructing an exception captures a full stack trace, which is measurably expensive — using `try`/`catch` to implement a normal, expected branch (like "is this the last element?" or "does this key exist?") turns a cheap conditional into a comparatively costly operation and obscures the code's actual intent behind exception-handling machinery. Exceptions should signal exceptional conditions, not everyday branching logic.

## Bad

```java
public boolean hasNext(Iterator<String> iterator) {
    try {
        String peek = iterator.next();
        pushback = peek;
        return true;
    } catch (NoSuchElementException e) {
        return false; // using an exception to ask a yes/no question
    }
}

public int parseOrDefault(String text, int defaultValue) {
    try {
        return Integer.parseInt(text);
    } catch (NumberFormatException e) {
        return defaultValue; // acceptable for genuinely untrusted input, but overused
    }
}

// The worst offender - using exceptions to break out of a loop as a "goto"
public Optional<Order> findFirstOverdue(List<Order> orders) {
    try {
        for (Order order : orders) {
            if (order.isOverdue()) {
                throw new FoundException(order);
            }
        }
    } catch (FoundException e) {
        return Optional.of(e.order());
    }
    return Optional.empty();
}
```

## Good

```java
public boolean hasNext(Iterator<String> iterator) {
    return iterator.hasNext(); // the API already gives you a direct predicate - use it
}

public Optional<Order> findFirstOverdue(List<Order> orders) {
    return orders.stream()
            .filter(Order::isOverdue)
            .findFirst(); // ordinary control flow, no exceptions involved
}
```

## When Catching For Control Is Reasonable

Parsing genuinely untrusted external input (`Integer.parseInt`, `LocalDate.parse`) is a legitimate use of `catch` for a fallback value, because the standard library gives you no cheaper way to validate the format up front. The distinction is: don't invent and throw your own exceptions purely to jump control flow within your own code.

## See Also

- [`anti-exception-for-flow-control`](anti-exception-for-flow-control.md) - Don't use exceptions for flow control
- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Validate arguments early and fail fast
- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Use streams for data transformation
- [`null-optional-return-type`](null-optional-return-type.md) - Use `Optional<T>` for return types
