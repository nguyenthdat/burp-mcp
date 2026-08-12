# api-final-classes-not-designed-for-inheritance

> Mark classes `final` unless designed and documented for inheritance

## Why It Matters

A non-final class is an implicit invitation to subclass it, and once a subclass exists anywhere, every future change to the base class's internal method calls must consider whether it breaks that subclass's overrides. As Joshua Bloch put it, a class must be "designed and documented for inheritance, or else prohibited" — leaving a class open by default without that deliberate design work sets a trap for both you and anyone who subclasses it later.

## Bad

```java
// Open by default, with no thought given to what happens if it's extended
public class RateLimiter {
    private int callsThisSecond = 0;
    private long windowStart = System.currentTimeMillis();

    public boolean tryAcquire() {
        refreshWindowIfNeeded();
        if (callsThisSecond >= 100) {
            return false;
        }
        callsThisSecond++;
        return true;
    }

    private void refreshWindowIfNeeded() {
        long now = System.currentTimeMillis();
        if (now - windowStart >= 1000) {
            windowStart = now;
            callsThisSecond = 0;
        }
    }
}

// A subclass someone writes later, unaware of internal call ordering
public class LoggingRateLimiter extends RateLimiter {
    @Override
    public boolean tryAcquire() {
        boolean result = super.tryAcquire();
        log.info("acquire attempt: {}", result);
        return result;
    }
    // Looks safe today - but if RateLimiter's internals change to call
    // tryAcquire() from a new method, this override runs in unexpected places
}
```

## Good

```java
public final class RateLimiter {
    private int callsThisSecond = 0;
    private long windowStart = System.currentTimeMillis();

    public boolean tryAcquire() {
        refreshWindowIfNeeded();
        if (callsThisSecond >= 100) {
            return false;
        }
        callsThisSecond++;
        return true;
    }

    private void refreshWindowIfNeeded() {
        long now = System.currentTimeMillis();
        if (now - windowStart >= 1000) {
            windowStart = now;
            callsThisSecond = 0;
        }
    }
}

// Extension is achieved through composition instead - no fragile coupling
public final class LoggingRateLimiter {
    private final RateLimiter delegate;

    public LoggingRateLimiter(RateLimiter delegate) {
        this.delegate = delegate;
    }

    public boolean tryAcquire() {
        boolean result = delegate.tryAcquire();
        log.info("acquire attempt: {}", result);
        return result;
    }
}
```

## Deliberately Designing for Inheritance

```java
/**
 * Base class for report exporters. Subclasses must implement
 * {@link #renderBody} and MUST NOT override {@link #export}, which
 * guarantees header/footer are always applied around the body.
 */
public abstract class ReportExporter {
    public final String export(Report report) {
        return header() + renderBody(report) + footer();
    }

    protected abstract String renderBody(Report report);

    protected String header() { return "=== Report ===\n"; }
    protected String footer() { return "=== End ===\n"; }
}
```

Here the class is explicitly documented as an extension point, the template method (`export`) is `final` to protect the invariant, and only the intended extension hook (`renderBody`) is overridable.

## See Also

- [`api-composition-over-inheritance`](api-composition-over-inheritance.md) - The default alternative to subclassing
- [`api-interface-default-methods`](api-interface-default-methods.md) - Sharing behavior without exposing a subclassable base
- [`api-sealed-closed-hierarchy`](api-sealed-closed-hierarchy.md) - Closing a hierarchy to a known, finite set of subclasses
- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Documenting the extension contract when inheritance is intended
