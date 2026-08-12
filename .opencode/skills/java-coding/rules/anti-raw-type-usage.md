# anti-raw-type-usage

> Don't use raw generic types

## Why It Matters

Using a raw type (`List` instead of `List<String>`) throws away all of the generics compiler checks, silently allowing incompatible objects into a collection and deferring the resulting `ClassCastException` to whatever line happens to read the value back out - often nowhere near where the bad element was inserted. Raw types exist only for backward compatibility with pre-Java-5 code and should never appear in new code.

## Bad

```java
List names = new ArrayList(); // Raw type - no compile-time element checking
names.add("Alice");
names.add(42); // Compiles fine, silently wrong

for (Object o : names) {
  String name = (String) o; // ClassCastException here, far from the real bug
  System.out.println(name.toUpperCase());
}

// Raw types in method signatures spread the problem to every caller
public void printAll(List items) {
  for (Object item : items) {
    System.out.println(item);
  }
}
```

## Good

```java
List<String> names = new ArrayList<>();
names.add("Alice");
names.add(42); // Compile error - caught immediately, not at runtime

for (String name : names) {
  System.out.println(name.toUpperCase());
}

public void printAll(List<?> items) { // Unbounded wildcard if the type truly doesn't matter
  for (Object item : items) {
    System.out.println(item);
  }
}
```

## When Interop with Legacy Code Requires It

```java
// Only acceptable when bridging to a pre-generics library API you don't control,
// and even then, isolate it behind a typed wrapper immediately.
@SuppressWarnings("unchecked")
// WHY: LegacyXmlParser predates generics and always returns homogeneous
// List<String> in practice; documented in its Javadoc.
public List<String> parseNames(LegacyXmlParser parser) {
  List raw = parser.parseElements(); // Legacy API returns a raw List
  return (List<String>) raw;
}
```

## What the Compiler Catches Instead

```java
// -Xlint:rawtypes flags every raw type usage at compile time
javac -Xlint:rawtypes -Werror Main.java
```

## See Also

- [`type-avoid-raw-types`](type-avoid-raw-types.md) - The positive rule this anti-pattern violates
- [`type-avoid-unchecked-cast`](type-avoid-unchecked-cast.md) - The unchecked casts raw types force you into
- [`type-bounded-wildcards-pecs`](type-bounded-wildcards-pecs.md) - The correct alternative when the exact type parameter is unknown
- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - Enabling `-Xlint:rawtypes -Werror` to catch this at compile time
