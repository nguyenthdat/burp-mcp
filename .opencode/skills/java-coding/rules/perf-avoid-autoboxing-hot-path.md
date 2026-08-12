# perf-avoid-autoboxing-hot-path

> Avoid autoboxing primitives in hot paths

## Why It Matters

Autoboxing converts a primitive like `int` into an `Integer` object on the heap, adding allocation, an extra pointer indirection, and eligibility for garbage collection. In a hot loop that runs millions of times, this turns cheap register arithmetic into heap churn and can dominate the profile. It also introduces subtle bugs, since boxed types compare by reference with `==` outside the cached `Integer` range (-128 to 127).

## Bad

```java
public long sumSquares(int[] values) {
    Long total = 0L;               // Boxed accumulator - reboxed every iteration
    for (int v : values) {
        total += (long) v * v;     // Unbox, add, rebox on every step
    }
    return total;
}

public Map<Integer, Integer> countOccurrences(int[] values) {
    Map<Integer, Integer> counts = new HashMap<>();  // Every key/value is a boxed Integer
    for (int v : values) {
        counts.merge(v, 1, Integer::sum);  // Boxing pressure on a hot counting loop
    }
    return counts;
}
```

## Good

```java
public long sumSquares(int[] values) {
    long total = 0L;                // Primitive accumulator, no boxing
    for (int v : values) {
        total += (long) v * v;
    }
    return total;
}

public int[] countOccurrences(int[] values, int maxValue) {
    int[] counts = new int[maxValue + 1];  // Primitive array instead of boxed map
    for (int v : values) {
        counts[v]++;
    }
    return counts;
}
```

## When Boxing Is Unavoidable

```java
// Generic collections require reference types - box at the boundary,
// not inside the hot inner loop.
List<Integer> results = new ArrayList<>(values.length);
for (int v : values) {
    int computed = compute(v);   // Do the hot work with primitives
    results.add(computed);       // Box once, only when storing the final result
}
```

## See Also

- [`perf-primitive-arrays-hot-path`](perf-primitive-arrays-hot-path.md) - Use primitive arrays for large numeric hot-path data
- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`coll-primitive-streams-hot-path`](coll-primitive-streams-hot-path.md) - Prefer primitive streams to avoid boxing overhead
- [`null-optional-primitive`](null-optional-primitive.md) - Use `OptionalInt`/`OptionalLong`/`OptionalDouble` to avoid boxing
