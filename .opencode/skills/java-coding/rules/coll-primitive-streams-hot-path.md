# coll-primitive-streams-hot-path

> Use primitive streams (`IntStream`, etc.) on hot paths

## Why It Matters

`Stream<Integer>`, `Stream<Long>`, and `Stream<Double>` box every element, which allocates a wrapper object per value and adds pointer-chasing overhead for arithmetic. `IntStream`, `LongStream`, and `DoubleStream` operate on unboxed primitives throughout the pipeline, avoiding that allocation entirely - a significant difference on hot paths processing large volumes of numeric data.

## Bad

```java
// Stream<Integer> boxes every element for a purely numeric computation
List<Integer> readings = sensorReadings();
int sum = readings.stream()
    .map(r -> r * 2) // boxing on every map invocation
    .reduce(0, Integer::sum); // unboxing/reboxing again

// Autoboxing inside a hot aggregation loop
double average = readings.stream()
    .mapToDouble(Integer::doubleValue) // at least converts, but source is still boxed
    .average()
    .orElse(0.0);

// Generating a boxed range for a tight numeric loop
List<Integer> indices = new ArrayList<>();
for (int i = 0; i < 1_000_000; i++) {
    indices.add(i); // boxes a million Integers just to iterate
}
indices.forEach(i -> process(i));
```

## Good

```java
// IntStream avoids boxing for the entire pipeline
int[] readings = sensorReadingsArray();
int sum = IntStream.of(readings)
    .map(r -> r * 2)
    .sum();

// mapToInt/mapToLong/mapToDouble convert once at the boundary, then stay primitive
double average = IntStream.of(readings)
    .average()
    .orElse(0.0);

// IntStream.range avoids boxing entirely for index-based iteration
IntStream.range(0, 1_000_000)
    .forEach(this::process);
```

## When the Boxed Stream Is Fine

For small collections, one-off computations, or code far from any hot path, the readability of a regular `Stream<Integer>` combined with `Collectors` is usually worth more than the micro-optimization of a primitive stream. Reserve primitive streams for code that profiling has shown to be allocation-sensitive or that runs over large numeric datasets.

```java
// Small, infrequent - boxed stream is perfectly acceptable here
List<Integer> topScores = leaderboard.stream()
    .map(Player::score)
    .sorted(Comparator.reverseOrder())
    .limit(3)
    .toList();
```

## See Also

- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - The general autoboxing performance concern this specializes
- [`coll-stream-vs-loop`](coll-stream-vs-loop.md) - When a primitive array loop beats even a primitive stream
- [`null-optional-primitive`](null-optional-primitive.md) - `OptionalInt`/`OptionalDouble` avoid boxing in the same spirit
- [`coll-stream-parallel-caution`](coll-stream-parallel-caution.md) - Primitive streams parallelize more predictably
