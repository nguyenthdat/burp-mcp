# perf-jmh-benchmarking

> Use JMH for reliable microbenchmarks

## Why It Matters

Hand-rolled benchmarks using `System.nanoTime()` around a loop are routinely wrong: the JIT can eliminate dead code, constant-fold results, hoist invariants out of the loop, or never warm up to steady-state performance, all of which produce numbers that don't reflect production behavior. JMH (Java Microbenchmark Harness) exists specifically to defeat these pitfalls with proper warmup iterations, forked JVMs, and dead-code-elimination guards, giving results you can actually trust and compare.

## Bad

```java
public class ManualBenchmark {
    public static void main(String[] args) {
        List<Integer> list = new ArrayList<>();
        for (int i = 0; i < 100_000; i++) {
            list.add(i);
        }

        long start = System.nanoTime();  // No warmup - JIT hasn't kicked in yet
        long sum = 0;
        for (int i = 0; i < list.size(); i++) {
            sum += list.get(i);  // Dead-store: result never used, JIT may eliminate the loop
        }
        long elapsed = System.nanoTime() - start;
        System.out.println("Took " + elapsed + "ns");  // sum is discarded
    }
}
```

## Good

```java
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.NANOSECONDS)
@State(Scope.Benchmark)
@Warmup(iterations = 5, time = 1)
@Measurement(iterations = 5, time = 1)
@Fork(2)
public class ListSumBenchmark {

    private List<Integer> list;

    @Setup(Level.Trial)
    public void setup() {
        list = new ArrayList<>();
        for (int i = 0; i < 100_000; i++) {
            list.add(i);
        }
    }

    @Benchmark
    public long sumList(Blackhole blackhole) {
        long sum = 0;
        for (int value : list) {
            sum += value;
        }
        blackhole.consume(sum);  // Prevents the JIT from eliminating the computation
        return sum;
    }
}

// build.gradle.kts
// plugins { id("me.champeau.jmh") version "0.7.2" }
// jmh { warmupIterations = 5; iterations = 5; fork = 2 }
```

## Reading Results Correctly

```
Benchmark                     Mode  Cnt    Score    Error  Units
ListSumBenchmark.sumList      avgt   10  842.113 ±  12.4   ns/op
```

Always check the error margin (`±`), run on a quiet machine or dedicated CI runner, and compare against a baseline benchmark run on the same hardware - never trust a single number in isolation.

## See Also

- [`perf-profile-before-optimizing`](perf-profile-before-optimizing.md) - Profile before optimizing
- [`perf-avoid-premature-optimization`](perf-avoid-premature-optimization.md) - Don't optimize before profiling
- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Avoid autoboxing primitives in hot paths
