# type-avoid-raw-types

> Avoid raw types; always parameterize generics

## Why It Matters

Raw types (`List` instead of `List<String>`) disable compile-time type checking entirely, turning what would be a compile error into a `ClassCastException` at runtime, often far away from the code that caused it. Raw types exist only for backward compatibility with pre-Java-5 code; new code should never introduce them.

## Bad

```java
// Raw type - no compile-time checking of element type
List names = new ArrayList();
names.add("Alice");
names.add(42); // compiles! wrong type silently added

for (Object o : names) {
    String s = (String) o; // ClassCastException at runtime on the Integer
    System.out.println(s.toUpperCase());
}

// Raw type in a method signature - defeats the purpose of generics
public void printAll(Collection c) {
    for (Object o : c) {
        System.out.println(o);
    }
}
```

## Good

```java
// Parameterized - compiler rejects the wrong type immediately
List<String> names = new ArrayList<>();
names.add("Alice");
names.add(42); // compile error: incompatible types

for (String s : names) {
    System.out.println(s.toUpperCase()); // no cast, no risk of CCE
}

// Use an unbounded wildcard when the element type truly doesn't matter
public void printAll(Collection<?> c) {
    for (Object o : c) {
        System.out.println(o);
    }
}
```

## Raw Type vs. Unbounded Wildcard

`List` (raw) and `List<?>` (unbounded wildcard) look similar but behave very differently:

```java
List<?> unknown = new ArrayList<String>();
unknown.add("x"); // compile error: can't add anything except null

List raw = new ArrayList<String>();
raw.add("x"); // compiles, but corrupts the list's true element type
```

Prefer `List<?>` whenever you need a container-agnostic parameter - it keeps type safety while raw types abandon it. The only legitimate uses of raw types today are reflection-adjacent code (`class` literals like `List.class`) and interfacing with pre-generics legacy libraries you cannot change.

## See Also

- [`type-avoid-unchecked-cast`](type-avoid-unchecked-cast.md) - Raw types are the most common source of unchecked cast warnings
- [`type-bounded-wildcards-pecs`](type-bounded-wildcards-pecs.md) - Use wildcards instead of raw types for flexible APIs
- [`anti-raw-type-usage`](anti-raw-type-usage.md) - Broader catalogue of raw-type anti-patterns
