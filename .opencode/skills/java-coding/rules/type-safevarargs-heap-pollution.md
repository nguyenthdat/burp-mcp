# type-safevarargs-heap-pollution

> Guard varargs generics with a correct `@SafeVarargs`

## Why It Matters

Varargs methods with a generic parameter type (`T... args`) implicitly create a generic array under the hood, which can cause "heap pollution" - an array whose reified type does not match its declared generic type - if the method stores elements into that array or exposes it. `@SafeVarargs` is a promise to callers and the compiler that the method does not perform any unsafe operations on the varargs array; applying it to a method that actually does is worse than not using it at all, since it silences the warning that would have caught the bug.

## Bad

```java
public class ListBuilder {
    // Dangerous: exposes the varargs array to the caller, unsuppressed
    public static <T> T[] toArray(T... elements) {
        return elements; // the array's runtime type may not match T[] the caller expects
    }

    // Falsely marked @SafeVarargs while actually leaking the array
    @SafeVarargs
    public static <T> T[] dangerousLeak(T... elements) {
        Object[] leaked = elements; // aliasing the generic array
        leaked[0] = "surprise";     // heap pollution: wrong type written into T[]
        return elements;
    }
}

String[] names = ListBuilder.toArray("a", "b"); // may throw ArrayStoreException
                                                 // or ClassCastException downstream
```

## Good

```java
public class ListBuilder {
    // Safe: only reads from the varargs array, never stores into it or leaks a reference
    @SafeVarargs
    public static <T> List<T> of(T... elements) {
        return List.copyOf(Arrays.asList(elements)); // defensive copy into a real List
    }
}

List<String> names = ListBuilder.of("a", "b"); // safe, no array exposed
```

## Rules for a Correct `@SafeVarargs`

`@SafeVarargs` is only valid on `static`, `final`, `private`, or (Java 9+) constructor methods, since these cannot be overridden with a different, unsafe implementation. Before applying it, verify:

1. The method never stores anything into the varargs array (no heap pollution).
2. The method never returns or otherwise exposes the varargs array to untrusted code (no reference leak that lets a caller reintroduce the array's mutable, erasure-unsafe view).
3. If both conditions hold, copy the array's contents into a proper collection (as `List.copyOf` does above) rather than passing the raw array further along.

If a method cannot satisfy both conditions, do not annotate it with `@SafeVarargs` - accept the "possible heap pollution" warning as a signal to redesign the method (e.g. accept a `List<T>` parameter instead of varargs).

## See Also

- [`type-generic-array-avoid`](type-generic-array-avoid.md) - Varargs generics are one of the main sources of generic-array problems
- [`type-avoid-unchecked-cast`](type-avoid-unchecked-cast.md) - The same "narrow, documented, justified" discipline applies here
- [`api-builder-complex-construction`](api-builder-complex-construction.md) - Varargs factory methods are common in builder-style APIs
