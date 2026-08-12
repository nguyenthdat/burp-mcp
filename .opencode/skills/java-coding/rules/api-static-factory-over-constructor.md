# api-static-factory-over-constructor

> Use static factory methods for readability and caching

## Why It Matters

Constructors must share the class's name, so overloaded constructors that differ only in argument types or order are indistinguishable at the call site, while a static factory method can carry a name that explains exactly what it produces. Factories also gain abilities constructors never have: returning a cached instance instead of allocating, returning a subtype or interface implementation, and validating or transforming arguments before an object exists.

## Bad

```java
public class Connection {
    private final String host;
    private final int port;
    private final boolean useTls;

    public Connection(String host, int port) {
        this(host, port, false);
    }

    public Connection(String host, int port, boolean useTls) {
        this.host = host;
        this.port = port;
        this.useTls = useTls;
    }
}

// Ambiguous at the call site - what does "true" mean here?
Connection conn = new Connection("db.internal", 5432, true);

// Every call allocates, even for the extremely common default case
Connection localhost = new Connection("localhost", 5432);
```

## Good

```java
public final class Connection {
    private static final Connection LOCALHOST = new Connection("localhost", 5432, false);

    private final String host;
    private final int port;
    private final boolean useTls;

    private Connection(String host, int port, boolean useTls) {
        this.host = host;
        this.port = port;
        this.useTls = useTls;
    }

    public static Connection of(String host, int port) {
        return new Connection(host, port, false);
    }

    public static Connection secure(String host, int port) {
        return new Connection(host, port, true); // name documents intent
    }

    public static Connection localhost() {
        return LOCALHOST; // cached - no allocation for the common case
    }
}

// Self-documenting, and the common case is free
Connection secureDb = Connection.secure("db.internal", 5432);
Connection local = Connection.localhost();
```

## Returning an Interface Type

```java
public interface Cache<K, V> {
    V get(K key);
    void put(K key, V value);

    static <K, V> Cache<K, V> boundedLru(int maxSize) {
        return new LruCache<>(maxSize); // implementation detail hidden from callers
    }

    static <K, V> Cache<K, V> unbounded() {
        return new UnboundedCache<>();
    }
}

// Callers depend only on the interface; the concrete class can change freely
Cache<String, User> userCache = Cache.boundedLru(1000);
```

## When a Public Constructor Is Fine

Simple, unambiguous value types with a single obvious construction path (many records fall in this category) rarely benefit from a factory — reserve factories for cases with caching, subtype selection, validation, or naming ambiguity to solve.

## See Also

- [`api-avoid-telescoping-constructors`](api-avoid-telescoping-constructors.md) - The overload-explosion problem factories help avoid
- [`api-builder-complex-construction`](api-builder-complex-construction.md) - The next step up when a factory's parameter list grows large
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Hiding constructors entirely behind a factory
- [`conc-holder-idiom-lazy-singleton`](conc-holder-idiom-lazy-singleton.md) - A specific caching factory pattern for lazy singletons
