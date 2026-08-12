# type-bounded-wildcards-pecs

> Use bounded wildcards (PECS: producer-extends, consumer-super)

## Why It Matters

APIs typed with exact generic parameters are needlessly rigid: a `List<Integer>` cannot be passed where `List<Number>` is expected, even though it is perfectly safe to read from. Bounded wildcards let library methods accept the widest possible range of caller types without sacrificing type safety, which is why the JDK collections APIs (`Collections.copy`, `addAll`) all use them.

## Bad

```java
// Only accepts List<Number> exactly - rejects List<Integer>, List<Double>
public static double sum(List<Number> numbers) {
    double total = 0;
    for (Number n : numbers) {
        total += n.doubleValue();
    }
    return total;
}

// Only accepts a destination of exactly Object - can't pass List<Number>
public static void addIntegers(List<Object> dest, List<Integer> source) {
    dest.addAll(source); // won't even compile against List<Number>
}

List<Integer> ints = List.of(1, 2, 3);
sum(ints); // compile error: List<Integer> is not List<Number>
```

## Good

```java
// PECS: producer of T -> use "? extends T" (we only read numbers out)
public static double sum(List<? extends Number> numbers) {
    double total = 0;
    for (Number n : numbers) {
        total += n.doubleValue();
    }
    return total;
}

// PECS: consumer of T -> use "? super T" (we only write integers in)
public static void addIntegers(List<? super Integer> dest, List<Integer> source) {
    dest.addAll(source);
}

List<Integer> ints = List.of(1, 2, 3);
sum(ints);                     // now compiles
addIntegers(new ArrayList<Number>(), ints); // now compiles
```

## Remembering PECS

- **Producer Extends**: if the parameter only produces (you read `T` values out of it), use `? extends T`.
- **Consumer Super**: if the parameter only consumes (you write `T` values into it), use `? super T`.
- If a parameter both produces and consumes, use an exact type (no wildcard) - this is common for something like a `Comparator<T>` used with `? super T` because comparators consume `T` and produce an `int`.
- Never use a wildcard for a return type; it forces callers to deal with `?` and provides no benefit.

## See Also

- [`type-avoid-raw-types`](type-avoid-raw-types.md) - Never fall back to raw types to dodge wildcard rules
- [`type-generic-method-inference`](type-generic-method-inference.md) - Let inference pick the captured wildcard type
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Wildcards widen accepted input without widening the public contract
