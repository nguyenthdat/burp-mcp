# api-composition-over-inheritance

> Favor composition over inheritance

## Why It Matters

Inheritance couples a subclass to its superclass's implementation details, not just its interface, so an "innocent" change to the base class can silently break every subclass without any of them changing a line of code. Composition instead exposes a stable, explicit dependency through an interface, making the relationship between classes visible, replaceable, and testable without fragile assumptions about internal call sequences.

## Bad

```java
// Extending a concrete collection ties us to its internal implementation
public class InstrumentedHashSet<E> extends HashSet<E> {
    private int addCount = 0;

    @Override
    public boolean add(E e) {
        addCount++;
        return super.add(e);
    }

    @Override
    public boolean addAll(Collection<? extends E> c) {
        addCount += c.size();
        return super.addAll(c); // HashSet.addAll calls add() internally,
                                 // so addCount is double-counted - a hidden
                                 // implementation detail we don't control
    }

    public int getAddCount() { return addCount; }
}

var set = new InstrumentedHashSet<String>();
set.addAll(List.of("a", "b", "c"));
System.out.println(set.getAddCount()); // 6, not 3 - broke silently
```

## Good

```java
// Wrap a Set instead of extending one; only the interface is a dependency
public class InstrumentedSet<E> implements Set<E> {
    private final Set<E> delegate;
    private int addCount = 0;

    public InstrumentedSet(Set<E> delegate) {
        this.delegate = delegate;
    }

    @Override
    public boolean add(E e) {
        addCount++;
        return delegate.add(e);
    }

    @Override
    public boolean addAll(Collection<? extends E> c) {
        addCount += c.size();
        return delegate.addAll(c); // one well-defined effect, no double counting
    }

    public int getAddCount() { return addCount; }

    // Remaining Set<E> methods delegate directly
    @Override public int size() { return delegate.size(); }
    @Override public boolean isEmpty() { return delegate.isEmpty(); }
    @Override public boolean contains(Object o) { return delegate.contains(o); }
    @Override public Iterator<E> iterator() { return delegate.iterator(); }
    @Override public Object[] toArray() { return delegate.toArray(); }
    @Override public <T> T[] toArray(T[] a) { return delegate.toArray(a); }
    @Override public boolean remove(Object o) { return delegate.remove(o); }
    @Override public boolean containsAll(Collection<?> c) { return delegate.containsAll(c); }
    @Override public boolean retainAll(Collection<?> c) { return delegate.retainAll(c); }
    @Override public boolean removeAll(Collection<?> c) { return delegate.removeAll(c); }
    @Override public void clear() { delegate.clear(); }
}

var set = new InstrumentedSet<String>(new HashSet<>());
set.addAll(List.of("a", "b", "c"));
System.out.println(set.getAddCount()); // 3, exactly what we intended
```

## When Inheritance Is Appropriate

Inheritance is safe within a true "is-a" relationship where the subclass is designed against a documented, stable base class contract (e.g. extending an abstract class specifically built for extension, such as `AbstractList`). If you cannot state the invariant the base class guarantees to subclasses, prefer composition.

## See Also

- [`api-final-classes-not-designed-for-inheritance`](api-final-classes-not-designed-for-inheritance.md) - Closing off classes not meant to be extended
- [`api-interface-default-methods`](api-interface-default-methods.md) - Sharing behavior without a rigid base class
- [`anti-god-class`](anti-god-class.md) - How unchecked inheritance chains grow into god classes
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keeping delegation targets narrow and intentional
