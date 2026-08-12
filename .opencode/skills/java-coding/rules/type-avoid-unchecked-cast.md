# type-avoid-unchecked-cast

> Isolate unchecked casts behind a documented `@SuppressWarnings`

## Why It Matters

Unchecked casts are compile-time promises to the compiler that a generic type is correct, backed by nothing at runtime because of type erasure. Scattering them throughout a codebase without justification hides genuinely unsafe operations among incidental ones, making it impossible to audit which casts actually need scrutiny during a bug hunt.

## Bad

```java
public class Cache<K, V> {
    private final Map<K, Object> store = new HashMap<>();

    public void put(K key, V value) {
        store.put(key, value);
    }

    @SuppressWarnings("unchecked") // slapped on the whole method, no explanation
    public V get(K key) {
        Object raw = store.get(key);
        return (V) raw; // unchecked cast could hide real bugs elsewhere in this method
    }

    @SuppressWarnings("unchecked")
    public V getOrDefault(K key, V fallback) {
        // second unrelated unchecked cast buried in the same suppressed scope
        Object raw = store.get(key);
        if (raw == null) return fallback;
        List<Object> maybeList = (List<Object>) raw; // completely different risk, same suppression
        return (V) (Object) maybeList;
    }
}
```

## Good

```java
public class Cache<K, V> {
    private final Map<K, Object> store = new HashMap<>();

    public void put(K key, V value) {
        store.put(key, value);
    }

    public V get(K key) {
        Object raw = store.get(key);
        return castStoredValue(raw);
    }

    public V getOrDefault(K key, V fallback) {
        Object raw = store.get(key);
        return raw == null ? fallback : castStoredValue(raw);
    }

    // Single, narrowly-scoped, documented unchecked cast
    @SuppressWarnings("unchecked") // safe: put() only ever stores V instances for this key space
    private V castStoredValue(Object raw) {
        return (V) raw;
    }
}
```

## Rules for the Suppressed Scope

1. Apply `@SuppressWarnings("unchecked")` to the smallest possible scope - a local variable declaration or a tiny private helper method, never a whole class.
2. Always pair it with a comment explaining *why* the cast is actually safe (an invariant the compiler cannot see).
3. Never suppress warnings just to make a build pass; if you cannot explain why the cast is safe, the design likely needs a `Class<V>` token or a different data structure instead.

## See Also

- [`type-avoid-raw-types`](type-avoid-raw-types.md) - Raw types are the most common cause of unchecked cast warnings
- [`type-safevarargs-heap-pollution`](type-safevarargs-heap-pollution.md) - `@SafeVarargs` is the varargs-specific version of this same discipline
- [`type-generic-array-avoid`](type-generic-array-avoid.md) - Generic array creation is another erasure workaround needing similar care
