# null-optional-primitive

> Use `OptionalInt`/`OptionalLong`/`OptionalDouble` for primitives

## Why It Matters

`Optional<Integer>` boxes the primitive twice over: once into an `Integer`, and again by wrapping it in the `Optional` container object, causing two allocations for what should be a single stack-allocated primitive check. The primitive-specialized types (`OptionalInt`, `OptionalLong`, `OptionalDouble`) avoid the boxing entirely and are exactly what stream terminal operations like `IntStream.average()` already return.

## Bad

```java
public Optional<Integer> maxScore(List<Attempt> attempts) {
    return attempts.stream()
            .map(Attempt::score)      // boxes each int into Integer
            .max(Integer::compareTo); // Optional<Integer> - double indirection
}

Optional<Integer> best = maxScore(attempts);
if (best.isPresent()) {
    System.out.println(best.get()); // unboxing again
}
```

## Good

```java
public OptionalInt maxScore(List<Attempt> attempts) {
    return attempts.stream()
            .mapToInt(Attempt::score) // IntStream - no boxing
            .max();                   // OptionalInt
}

OptionalInt best = maxScore(attempts);
if (best.isPresent()) {
    System.out.println(best.getAsInt());
}

// Or functionally:
int shown = maxScore(attempts).orElse(0);
```

## Consistent Across The Primitive Family

```java
OptionalInt maxLength = names.stream().mapToInt(String::length).max();
OptionalLong totalBytes = files.stream().mapToLong(File::length).min();
OptionalDouble averagePrice = prices.stream().mapToDouble(Price::amount).average();
```

## Limitation To Be Aware Of

`OptionalInt`/`OptionalLong`/`OptionalDouble` do not have `map`/`flatMap`/`filter` like `Optional<T>` does — only `orElse`, `orElseGet`, `orElseThrow`, `ifPresent`, and `getAsX`. For richer chaining you must convert with `.stream()` or fall back to boxed `Optional<Integer>` deliberately when the transformation logic demands it.

## See Also

- [`null-optional-return-type`](null-optional-return-type.md) - Use `Optional<T>` for return types only
- [`coll-primitive-streams-hot-path`](coll-primitive-streams-hot-path.md) - Use primitive streams in hot paths
- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Avoid autoboxing in hot paths
