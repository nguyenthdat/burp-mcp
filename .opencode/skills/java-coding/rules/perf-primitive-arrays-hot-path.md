# perf-primitive-arrays-hot-path

> Use primitive arrays for large numeric hot-path data

## Why It Matters

An `ArrayList<Double>` or `Double[]` stores boxed objects scattered across the heap, so iterating it means chasing pointers and paying cache misses in addition to boxing overhead. A `double[]` is a single contiguous block of memory the CPU can prefetch efficiently, with no per-element object header. For numeric-heavy workloads - vector math, signal processing, large in-memory datasets - this difference in memory layout routinely accounts for a multi-times speedup.

## Bad

```java
public class PriceSeries {
    private final List<Double> prices = new ArrayList<>();  // Boxed, non-contiguous

    public void add(double price) {
        prices.add(price);  // Autoboxes every value
    }

    public double average() {
        double sum = 0;
        for (Double price : prices) {  // Unboxes every value, pointer-chases the list
            sum += price;
        }
        return sum / prices.size();
    }
}
```

## Good

```java
public class PriceSeries {
    private double[] prices = new double[64];  // Contiguous primitive storage
    private int size = 0;

    public void add(double price) {
        if (size == prices.length) {
            prices = Arrays.copyOf(prices, prices.length * 2);  // Grow like ArrayList would
        }
        prices[size++] = price;
    }

    public double average() {
        double sum = 0;
        for (int i = 0; i < size; i++) {
            sum += prices[i];  // No boxing, sequential memory access
        }
        return sum / size;
    }
}
```

## When Boxed Collections Are the Better Trade

```java
// Small collections, or ones that need generics, null handling, or
// interoperate with the Collections framework - the object overhead is
// negligible and the API convenience wins.
Map<String, List<Double>> pricesByTicker = new HashMap<>();
pricesByTicker.computeIfAbsent("AAPL", k -> new ArrayList<>()).add(190.5);
```

## See Also

- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Avoid autoboxing primitives in hot paths
- [`coll-primitive-streams-hot-path`](coll-primitive-streams-hot-path.md) - Prefer primitive streams to avoid boxing overhead
- [`perf-collection-sizing`](perf-collection-sizing.md) - Size collections up front when the count is known
