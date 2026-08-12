# perf-avoid-unnecessary-object-creation

> Avoid unnecessary object creation in hot paths

## Why It Matters

Every object allocation is cheap in isolation, but a hot path that allocates on every iteration - a new formatter, a new regex `Pattern`, a throwaway array - turns into constant garbage-collector pressure at scale. Reusing immutable or stateless objects, or hoisting allocation out of a loop, removes work the JVM would otherwise repeat millions of times without changing behavior.

## Bad

```java
public boolean isValidEmail(String email) {
    Pattern pattern = Pattern.compile("^[\\w.+-]+@[\\w-]+\\.[a-zA-Z]{2,}$");  // Compiled every call
    return pattern.matcher(email).matches();
}

public String formatAmount(double amount) {
    DecimalFormat format = new DecimalFormat("#,##0.00");  // Allocated on every call
    return format.format(amount);
}

public List<Result> processAll(List<Request> requests) {
    List<Result> results = new ArrayList<>();
    for (Request request : requests) {
        int[] scratch = new int[1024];  // New scratch buffer every iteration
        results.add(process(request, scratch));
    }
    return results;
}
```

## Good

```java
private static final Pattern EMAIL_PATTERN =
    Pattern.compile("^[\\w.+-]+@[\\w-]+\\.[a-zA-Z]{2,}$");  // Compiled once, reused

public boolean isValidEmail(String email) {
    return EMAIL_PATTERN.matcher(email).matches();
}

public String formatAmount(double amount) {
    // DecimalFormat is not thread-safe; use a ThreadLocal or a fresh
    // instance per thread rather than a shared static field.
    return AMOUNT_FORMAT.get().format(amount);
}

private static final ThreadLocal<DecimalFormat> AMOUNT_FORMAT =
    ThreadLocal.withInitial(() -> new DecimalFormat("#,##0.00"));

public List<Result> processAll(List<Request> requests) {
    List<Result> results = new ArrayList<>(requests.size());
    int[] scratch = new int[1024];  // Allocated once, reused across iterations
    for (Request request : requests) {
        results.add(process(request, scratch));
    }
    return results;
}
```

## See Also

- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Avoid autoboxing primitives in hot paths
- [`perf-collection-sizing`](perf-collection-sizing.md) - Size collections up front when the count is known
- [`perf-lazy-initialization-holder`](perf-lazy-initialization-holder.md) - Defer expensive initialization with lazy holders
- [`conc-avoid-shared-mutable-state`](conc-avoid-shared-mutable-state.md) - Avoid shared mutable state across threads
