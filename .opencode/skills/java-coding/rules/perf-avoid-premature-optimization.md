# perf-avoid-premature-optimization

> Don't optimize before profiling

## Why It Matters

Optimizing code before measuring where time actually goes almost always trades away readability and maintainability for a speedup that doesn't exist, or exists somewhere the application never spends meaningful time. Developers are notoriously bad at guessing hot spots; modern JITs, caches, and I/O costs make intuition unreliable. The result of premature optimization is usually convoluted code that is harder to change later, for no measured benefit.

## Bad

```java
public class OrderProcessor {
    // "Optimized" by hand-inlining and avoiding a helper method, before any
    // measurement showed this method was ever a bottleneck.
    public double computeTotal(List<OrderLine> lines) {
        double total = 0;
        int size = lines.size();
        for (int i = 0; i < size; i++) {
            OrderLine line = lines.get(i);
            double lineTotal = line.quantity() * line.unitPrice();
            // Manually unrolled "for speed" - adds complexity, no measured gain
            if (i + 1 < size) {
                OrderLine next = lines.get(i + 1);
                total += lineTotal + next.quantity() * next.unitPrice();
                i++;
            } else {
                total += lineTotal;
            }
        }
        return total;
    }
}
```

## Good

```java
public class OrderProcessor {
    // Clear, obviously-correct code. Only revisited if profiling shows it matters.
    public double computeTotal(List<OrderLine> lines) {
        double total = 0;
        for (OrderLine line : lines) {
            total += line.quantity() * line.unitPrice();
        }
        return total;
    }
}
```

## The Right Order of Operations

```java
// 1. Write clear, correct code first.
// 2. Establish a performance requirement (e.g. "p99 checkout < 200ms").
// 3. Measure with a profiler (async-profiler, JFR) or a JMH benchmark
//    under realistic load to find the actual hot spot.
// 4. Optimize only the measured hot spot, and re-measure to confirm the gain.
// 5. Keep the simpler version if the "optimization" doesn't show a real,
//    reproducible improvement.
```

## See Also

- [`perf-profile-before-optimizing`](perf-profile-before-optimizing.md) - Profile before optimizing
- [`perf-jmh-benchmarking`](perf-jmh-benchmarking.md) - Use JMH for reliable microbenchmarks
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API small and intentional
