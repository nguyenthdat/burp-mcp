# name-type-param-single-letter

> Use single uppercase letters for generic type parameters

## Why It Matters

Single-letter type parameters (`T`, `E`, `K`, `V`, `R`) are an established Java convention that lets readers instantly distinguish a type variable from a concrete class name, since concrete classes are always multi-letter `PascalCase`. Using descriptive multi-letter names for type parameters (`TValue`, `ItemType`) blurs that distinction and fights the JDK's own generics (`List<E>`, `Map<K, V>`, `Function<T, R>`), making unfamiliar APIs harder to read by analogy.

## Bad

```java
public class Box<BoxContentType> {  // reads like a concrete class name
    private BoxContentType content;

    public BoxContentType get() {
        return content;
    }
}

public interface Transformer<InputType, OutputType> {  // verbose, non-idiomatic
    OutputType transform(InputType input);
}
```

## Good

```java
public class Box<T> {
    private T content;

    public T get() {
        return content;
    }
}

public interface Transformer<T, R> {
    R transform(T input);
}
```

## Conventional Letters

Java establishes de facto meanings for common single letters, and deviating from them (e.g., using `V` for a generic "value" that is not map-related) creates unnecessary friction:

```java
// T - Type (generic, general purpose)
// E - Element (collections)
// K - Key
// V - Value
// N - Number
// R - Result (often a method's return type)
// U, S - additional generic types when T is already used

public interface Repository<T, ID> {
    Optional<T> findById(ID id);
}

public class Pair<K, V> {
    private final K key;
    private final V value;

    public Pair(K key, V value) {
        this.key = key;
        this.value = value;
    }
}
```

When a single letter would genuinely be ambiguous across several unrelated type parameters in one declaration, adding a numeric suffix (`T1`, `T2`) is acceptable over inventing a descriptive multi-word name, though such signatures are usually a sign the API itself should be simplified.

## See Also

- [`type-bounded-wildcards-pecs`](type-bounded-wildcards-pecs.md) - Use bounded wildcards following PECS
- [`type-recursive-generic-bound`](type-recursive-generic-bound.md) - Model recursive generic bounds correctly
- [`name-classes-pascal`](name-classes-pascal.md) - Use PascalCase for classes, interfaces, enums, records
