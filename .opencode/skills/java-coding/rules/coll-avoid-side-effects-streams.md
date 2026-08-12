# coll-avoid-side-effects-streams

> Avoid side effects inside stream operations

## Why It Matters

Stream operations are meant to be stateless and side-effect-free; the pipeline's evaluation order, laziness, and parallelizability are all unspecified beyond the guarantees the API documents. Mutating external state from inside `map`/`filter`/`forEach` produces code that happens to work under the current sequential implementation but breaks silently (or non-deterministically) if the stream is later made parallel, reordered, or short-circuited.

## Bad

```java
// Mutating an external list from inside a stream - defeats the purpose of collect()
List<String> results = new ArrayList<>();
orders.stream()
    .filter(order -> order.total().compareTo(BigDecimal.ZERO) > 0)
    .forEach(order -> results.add(order.customer().name())); // side effect

// Mutating a shared counter - not thread-safe, breaks under .parallel()
int[] count = {0};
items.stream().forEach(item -> {
    if (item.isValid()) {
        count[0]++; // race condition if parallelized later
    }
});

// map() with a side-effecting function - hides an important state change
List<Integer> processed = values.stream()
    .map(v -> {
        auditLog.add("processed " + v); // side effect inside map
        return v * 2;
    })
    .toList();
```

## Good

```java
// Collect directly into the result - no external mutation
List<String> results = orders.stream()
    .filter(order -> order.total().compareTo(BigDecimal.ZERO) > 0)
    .map(order -> order.customer().name())
    .toList();

// Use a terminal operation designed for counting
long count = items.stream()
    .filter(Item::isValid)
    .count();

// Keep the side effect explicit and separate from the transformation
List<Integer> processed = values.stream()
    .map(v -> v * 2)
    .toList();
auditLog.addAll(values.stream().map(v -> "processed " + v).toList());
```

## `forEach` Is Not Always a Side Effect

`forEach` is appropriate as a genuine terminal action - printing, sending a message, invoking an external API - as long as each invocation is independent and doesn't mutate shared state that other stream elements also touch:

```java
// Fine: each call is independent, no shared mutable state
notifications.stream()
    .filter(Notification::isPending)
    .forEach(notificationService::send);
```

## See Also

- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Using streams for their intended declarative purpose
- [`coll-stream-parallel-caution`](coll-stream-parallel-caution.md) - Why side effects are especially dangerous under `.parallel()`
- [`conc-avoid-shared-mutable-state`](conc-avoid-shared-mutable-state.md) - The broader concurrency principle this rule is a special case of
- [`coll-collectors-toX`](coll-collectors-toX.md) - Collecting into the right terminal shape instead of mutating externally
