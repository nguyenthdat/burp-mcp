# coll-stream-vs-loop

> Choose streams vs loops based on readability and performance

## Why It Matters

Streams are not universally superior to loops - they add lambda allocation, boxing for non-primitive streams, and call-site indirection that a JIT-optimized loop avoids. The right default is to reach for a stream when the code is a straightforward transform/filter/reduce pipeline, and to reach for a loop when there's complex branching, early exits with multiple conditions, checked exceptions, or when profiling shows the stream overhead actually matters.

## Bad

```java
// Forcing a stream onto logic that needs multiple early exits and mutable state
Optional<Order> firstProblem = orders.stream()
    .filter(order -> {
        if (order.total().signum() < 0) {
            return true;
        }
        if (order.items().isEmpty()) {
            return true;
        }
        return order.customer() == null;
    })
    .findFirst();
// Reads awkwardly - the "real" logic is buried inside a filter predicate

// Using a loop for a simple one-line transformation, adding needless verbosity
List<String> upper = new ArrayList<>();
for (String name : names) {
    upper.add(name.toUpperCase());
}
```

## Good

```java
// Loop is clearer here - multiple named conditions, natural early return
Order firstProblem = null;
for (Order order : orders) {
    if (order.total().signum() < 0 || order.items().isEmpty() || order.customer() == null) {
        firstProblem = order;
        break;
    }
}

// Stream is clearer here - single expression, no intermediate state
List<String> upper = names.stream()
    .map(String::toUpperCase)
    .toList();
```

## Decision Guide

| Situation | Prefer |
|-----------|--------|
| Simple filter/map/reduce/collect pipeline | Stream |
| Multiple early exits or complex branching | Loop |
| Checked exceptions thrown per element | Loop (or a helper that wraps them) |
| Index needed alongside the value | Loop, or `IntStream.range` |
| Hot path where profiling shows lambda/boxing overhead | Loop, primitive arrays |
| Need to break out of nested iteration | Loop with labeled break |

## Checked Exceptions Inside Streams

Streams don't compose well with checked exceptions - lambdas can't declare `throws` for a checked type without wrapping. This alone is often reason enough to fall back to a loop:

```java
// Awkward: forced to wrap the checked exception
List<String> contents = paths.stream()
    .map(path -> {
        try {
            return Files.readString(path);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    })
    .toList();

// Clearer as a loop when the exception needs real handling per file
List<String> contents = new ArrayList<>();
for (Path path : paths) {
    try {
        contents.add(Files.readString(path));
    } catch (IOException e) {
        log.warn("Skipping unreadable file: {}", path, e);
    }
}
```

## See Also

- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - When a stream pipeline is the right tool
- [`coll-avoid-side-effects-streams`](coll-avoid-side-effects-streams.md) - Signs a stream is being forced onto imperative logic
- [`err-checked-vs-unchecked`](err-checked-vs-unchecked.md) - Exception handling trade-offs affecting this decision
- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Performance cost of streams on hot paths
