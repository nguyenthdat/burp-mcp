# conc-holder-idiom-lazy-singleton

> Use the initialization-on-demand holder idiom for lazy singletons

## Why It Matters

Lazily initializing a singleton with a naive `if (instance == null) instance = new Thing()` check is not thread-safe, and the common fix — synchronizing the whole accessor, or double-checked locking without `volatile` — either kills performance or is subtly broken. The initialization-on-demand holder idiom relies on the JVM's class-loading guarantees (a class is initialized lazily, exactly once, and initialization is synchronized by the classloader) to get lazy, thread-safe, lock-free singleton access for free.

## Bad

```java
public class ExpensiveResource {
    private static ExpensiveResource instance;

    public static ExpensiveResource getInstance() {
        // BAD: not thread-safe. Two threads can both see instance == null,
        // both construct a new ExpensiveResource, and one assignment wins,
        // silently discarding the other's work (and possibly its side effects).
        if (instance == null) {
            instance = new ExpensiveResource();
        }
        return instance;
    }

    private ExpensiveResource() {
        // expensive setup
    }
}
```

## Good

```java
public class ExpensiveResource {
    private ExpensiveResource() {
        // expensive setup
    }

    // The holder class is not loaded -- and ExpensiveResource is not
    // constructed -- until getInstance() is first called, at which point
    // the JVM guarantees exactly-once, thread-safe initialization.
    private static class Holder {
        static final ExpensiveResource INSTANCE = new ExpensiveResource();
    }

    public static ExpensiveResource getInstance() {
        return Holder.INSTANCE; // lock-free after the first call
    }
}
```

## Why This Beats Double-Checked Locking

Double-checked locking can be made correct with a `volatile` field, but it is easy to get wrong (missing `volatile` reintroduces a visibility bug where a thread can observe a partially-constructed object), and it still pays a memory-barrier cost on every read:

```java
// Correct, but more ceremony than the holder idiom and still not "free":
private static volatile ExpensiveResource instance;

public static ExpensiveResource getInstance() {
    ExpensiveResource result = instance;
    if (result == null) {
        synchronized (ExpensiveResource.class) {
            result = instance;
            if (result == null) {
                instance = result = new ExpensiveResource();
            }
        }
    }
    return result;
}
```

## When an `enum` Singleton Is Simpler

If laziness is not required, a single-element `enum` is the simplest thread-safe singleton in Java, since the JVM guarantees enum constants are constructed exactly once and it is inherently serialization-safe:

```java
public enum EagerResource {
    INSTANCE;

    void doWork() { /* ... */ }
}

// EagerResource.INSTANCE.doWork();
```

## See Also

- [`conc-immutable-thread-safety`](conc-immutable-thread-safety.md) - Make shared objects immutable for thread safety
- [`conc-atomic-over-lock`](conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - Static factory method patterns
