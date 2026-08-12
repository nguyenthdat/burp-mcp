# perf-lazy-initialization-holder

> Defer expensive initialization with lazy holders

## Why It Matters

Some resources - large caches, parsed configuration, cryptographic contexts - are expensive to build but not always needed on every application path. Eagerly initializing them in a static initializer or constructor slows down class loading and startup even when the resource is never touched. The initialization-on-demand holder idiom defers the cost to first use while relying on the JVM's class-loading guarantees for thread safety, without any explicit locking.

## Bad

```java
public class ReportEngine {
    // Built eagerly at class-load time, even if reports are never generated
    private static final TemplateCache CACHE = TemplateCache.loadAllFromDisk();

    public static String render(String templateName, Map<String, Object> data) {
        return CACHE.get(templateName).render(data);
    }
}

public class ExpensiveSingleton {
    private static ExpensiveSingleton instance;

    public static synchronized ExpensiveSingleton getInstance() {  // Lock on every call
        if (instance == null) {
            instance = new ExpensiveSingleton();
        }
        return instance;
    }
}
```

## Good

```java
public class ReportEngine {
    private static final class CacheHolder {
        // Only loaded the first time CacheHolder is referenced
        static final TemplateCache CACHE = TemplateCache.loadAllFromDisk();
    }

    public static String render(String templateName, Map<String, Object> data) {
        return CacheHolder.CACHE.get(templateName).render(data);
    }
}

public class ExpensiveSingleton {
    private ExpensiveSingleton() {
    }

    private static final class Holder {
        static final ExpensiveSingleton INSTANCE = new ExpensiveSingleton();
    }

    public static ExpensiveSingleton getInstance() {
        return Holder.INSTANCE;  // No synchronization; JVM guarantees safe, one-time init
    }
}
```

## Why the Holder Idiom Beats Double-Checked Locking

```java
// Double-checked locking requires a volatile field and careful memory-model
// reasoning to be correct, and still pays a volatile read on every call.
private static volatile ExpensiveSingleton instance;

public static ExpensiveSingleton getInstance() {
    ExpensiveSingleton local = instance;
    if (local == null) {
        synchronized (ExpensiveSingleton.class) {
            local = instance;
            if (local == null) {
                instance = local = new ExpensiveSingleton();
            }
        }
    }
    return local;
}
// The holder idiom achieves the same laziness and thread safety with no
// volatile field, no synchronized block, and no risk of getting it wrong.
```

## See Also

- [`conc-holder-idiom-lazy-singleton`](conc-holder-idiom-lazy-singleton.md) - Concurrency-focused treatment of the same idiom
- [`perf-avoid-premature-optimization`](perf-avoid-premature-optimization.md) - Don't optimize before profiling
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - Static factory methods over public constructors
