# coll-stream-parallel-caution

> Use parallel streams only after profiling, on splittable CPU-bound work

## Why It Matters

`parallel()` submits work to the common `ForkJoinPool`, which is shared process-wide with every other parallel stream and any `CompletableFuture` that doesn't specify its own executor. For small collections, I/O-bound work, or sources that don't split efficiently (like `LinkedList` or streams from I/O), the overhead of splitting, thread coordination, and merging routinely costs more than it saves - and it can starve unrelated parallel work elsewhere in the application.

## Bad

```java
// Parallelizing a small collection - overhead dwarfs any gain
List<Integer> smallList = List.of(1, 2, 3, 4, 5);
int sum = smallList.parallelStream()
    .mapToInt(Integer::intValue)
    .sum();

// Parallelizing I/O-bound work - blocks common ForkJoinPool threads on network calls
List<String> results = urls.parallelStream()
    .map(this::fetchOverHttp) // blocking I/O inside a parallel stream
    .toList();

// Parallelizing a LinkedList - poor spliterator characteristics, no real speedup
LinkedList<BigInteger> values = loadValues();
BigInteger product = values.parallelStream()
    .reduce(BigInteger.ONE, BigInteger::multiply);
```

## Good

```java
// Large, CPU-bound, well-splitting source (ArrayList/array) - parallel helps
List<BigInteger> values = loadLargeValueList(); // millions of elements, ArrayList
BigInteger product = values.parallelStream()
    .reduce(BigInteger.ONE, BigInteger::multiply);

// I/O-bound work belongs on a dedicated executor, not the common pool
List<String> results;
try (ExecutorService executor = Executors.newFixedThreadPool(20)) {
    List<Future<String>> futures = urls.stream()
        .map(url -> executor.submit(() -> fetchOverHttp(url)))
        .toList();
    results = futures.stream().map(this::getUnchecked).toList();
}

// Small or uncertain-size collections stay sequential
int sum = smallList.stream()
    .mapToInt(Integer::intValue)
    .sum();
```

## When Parallel Is Worth Considering

- The source is large (rule of thumb: hundreds of thousands of elements or more).
- The work per element is CPU-bound and roughly uniform (no blocking calls).
- The source splits well - arrays, `ArrayList`, `IntStream.range` - rather than `LinkedList` or `Iterator`-based sources.
- Profiling under realistic load actually shows a benefit; parallel streams should be an optimization applied after measurement, not a default.

## Isolating From the Common Pool

If parallel streams are genuinely warranted but must not compete with other common-pool consumers, submit them from within a custom `ForkJoinPool`:

```java
ForkJoinPool customPool = new ForkJoinPool(4);
try {
    BigInteger product = customPool.submit(() ->
        values.parallelStream().reduce(BigInteger.ONE, BigInteger::multiply)
    ).get();
} finally {
    customPool.shutdown();
}
```

## See Also

- [`coll-avoid-side-effects-streams`](coll-avoid-side-effects-streams.md) - Side effects become race conditions once a stream is parallelized
- [`conc-platform-threads-cpu`](conc-platform-threads-cpu.md) - When CPU-bound parallelism is the right tool at all
- [`conc-virtual-threads-io`](conc-virtual-threads-io.md) - The correct model for I/O-bound concurrent work instead of parallel streams
- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Primitive streams reduce per-element overhead, relevant when parallelizing
