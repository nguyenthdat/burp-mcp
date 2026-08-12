# type-generic-array-avoid

> Avoid generic array creation; use a `List` instead

## Why It Matters

Java forbids creating arrays of a generic type directly (`new T[10]` does not compile) because arrays are covariant and reified while generics are erased, and mixing the two lets a `ArrayStoreException` slip past the compiler undetected. Workarounds that force the issue with unchecked casts reintroduce exactly the unsafety generics were designed to prevent, so the idiomatic fix is to use `List<T>` wherever a generic array would have been used.

## Bad

```java
public class Stack<T> {
    // Doesn't compile: "generic array creation"
    // private T[] elements = new T[16];

    @SuppressWarnings("unchecked")
    private T[] elements = (T[]) new Object[16]; // unchecked cast to work around it
    private int size = 0;

    public void push(T item) {
        elements[size++] = item; // ArrayStoreException risk is hidden by erasure
    }

    public T pop() {
        return elements[--size];
    }
}

// Elsewhere: mixing generic arrays with covariance is exactly what breaks
Object[] objects = new Stack<String>().elements; // would let wrong types be stored
```

## Good

```java
public class Stack<T> {
    private final List<T> elements = new ArrayList<>();

    public void push(T item) {
        elements.add(item);
    }

    public T pop() {
        return elements.remove(elements.size() - 1);
    }

    public boolean isEmpty() {
        return elements.isEmpty();
    }
}
```

## When an Array Is Genuinely Required

Some APIs (e.g. interop with libraries expecting `T[]`, or performance-critical numeric code) do require an array. In that case, take a `Class<T>` token and use `java.lang.reflect.Array.newInstance`, and isolate the unchecked cast to that one spot:

```java
@SuppressWarnings("unchecked")
public static <T> T[] newArray(Class<T> type, int size) {
    return (T[]) java.lang.reflect.Array.newInstance(type, size);
}

Integer[] ints = newArray(Integer.class, 10);
```

Reach for `List<T>` first; only fall back to this pattern when an actual array type is required by an external contract.

## See Also

- [`type-avoid-unchecked-cast`](type-avoid-unchecked-cast.md) - The suppression discipline needed if an array truly must be created
- [`type-safevarargs-heap-pollution`](type-safevarargs-heap-pollution.md) - Varargs methods create generic arrays implicitly behind the scenes
- [`coll-choose-right-collection`](coll-choose-right-collection.md) - Picking the right `List` implementation once you've made this switch
