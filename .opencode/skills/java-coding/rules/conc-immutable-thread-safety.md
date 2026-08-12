# conc-immutable-thread-safety

> Make shared objects immutable for thread safety

## Why It Matters

An object with no mutable state after construction cannot exhibit a data race — there is nothing for concurrent threads to disagree about. Immutability turns thread safety from a synchronization problem you must get right on every access into a construction problem you get right once, and it composes safely: immutable objects can be freely shared, cached, and passed between virtual threads with zero locking.

## Bad

```java
// Mutable, shared configuration object with setters.
public class ServiceConfig {
    private int timeoutMillis;
    private String endpoint;

    public void setTimeoutMillis(int timeoutMillis) {
        this.timeoutMillis = timeoutMillis;
    }

    public void setEndpoint(String endpoint) {
        this.endpoint = endpoint;
    }

    public int getTimeoutMillis() { return timeoutMillis; }
    public String getEndpoint() { return endpoint; }
}

// Shared across many request-handling threads:
static final ServiceConfig CONFIG = new ServiceConfig();

void handleRequest() {
    // BAD: reading fields that another thread might be mutating right now
    // via setTimeoutMillis/setEndpoint, with no happens-before relationship
    // guaranteeing visibility of the latest values.
    call(CONFIG.getEndpoint(), CONFIG.getTimeoutMillis());
}
```

## Good

```java
// Immutable: all fields final, set once at construction, no setters.
public record ServiceConfig(int timeoutMillis, String endpoint) {
    public ServiceConfig {
        if (timeoutMillis <= 0) {
            throw new IllegalArgumentException("timeoutMillis must be positive");
        }
        Objects.requireNonNull(endpoint, "endpoint");
    }
}

// Safe to publish via a plain final field or volatile reference; readers
// always see a fully-constructed, internally consistent instance.
private static final ServiceConfig CONFIG = new ServiceConfig(5000, "https://api.example.com");

void handleRequest() {
    call(CONFIG.endpoint(), CONFIG.timeoutMillis()); // no lock needed, ever
}
```

## Updating "Configuration" Without Mutating It

When configuration genuinely needs to change at runtime, swap the entire immutable object atomically rather than mutating fields in place:

```java
private final AtomicReference<ServiceConfig> config =
        new AtomicReference<>(new ServiceConfig(5000, "https://api.example.com"));

void reload(ServiceConfig updated) {
    config.set(updated); // atomic publish; readers see either the old or new
                          // config in full, never a half-updated mix of fields
}

void handleRequest() {
    ServiceConfig current = config.get();
    call(current.endpoint(), current.timeoutMillis());
}
```

## What "Immutable" Actually Requires

All fields `final`, no exposed mutators, defensive copies of any mutable fields taken in on construction (or on return from accessors), and no reference to `this` escaping the constructor before it completes. A `record` gives you the first three for free but still requires you to defensively copy mutable components like `List` or `Date` in a compact constructor.

## See Also

- [`conc-avoid-shared-mutable-state`](conc-avoid-shared-mutable-state.md) - Avoid shared mutable state; prefer immutability/confinement
- [`api-immutable-by-default`](api-immutable-by-default.md) - Designing types to be immutable by default
- [`modern-records-immutable-data`](modern-records-immutable-data.md) - Using records for immutable data carriers
- [`null-defensive-copy`](null-defensive-copy.md) - Defensive copying of mutable fields and parameters
