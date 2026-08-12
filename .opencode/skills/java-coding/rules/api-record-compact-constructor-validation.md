# api-record-compact-constructor-validation

> Validate invariants in a record's compact constructor

## Why It Matters

A record's whole appeal is that it's a transparent, trustworthy data carrier — but that trust collapses if it can be constructed in an invalid state, like a negative age or an empty required string. The compact constructor is the single choke point every instance of a record must pass through, so validating there (rather than in a factory method that can be bypassed by calling `new` directly) guarantees the invariant holds for every instance, everywhere.

## Bad

```java
public record Range(int min, int max) {
    // No validation - min > max produces a nonsensical but constructible Range
}

Range invalid = new Range(10, 5); // silently accepted; bugs surface far downstream
if (invalid.min() <= invalid.max()) {
    // never true, but nothing stopped construction from getting here
}
```

## Good

```java
public record Range(int min, int max) {
    // Compact constructor: runs for every construction path, field assignment
    // happens implicitly afterward using these (possibly adjusted) parameters
    public Range {
        if (min > max) {
            throw new IllegalArgumentException(
                    "min (%d) must not exceed max (%d)".formatted(min, max));
        }
    }
}

Range valid = new Range(5, 10);   // fine
Range invalid = new Range(10, 5); // throws IllegalArgumentException immediately
```

## Normalizing Values in the Compact Constructor

```java
public record EmailAddress(String value) {
    public EmailAddress {
        Objects.requireNonNull(value, "value must not be null");
        if (!value.contains("@")) {
            throw new IllegalArgumentException("Invalid email address: " + value);
        }
        value = value.trim().toLowerCase(Locale.ROOT); // normalize before assignment
    }
}

EmailAddress e = new EmailAddress("  Alice@Example.COM  ");
System.out.println(e.value()); // alice@example.com
```

The compact constructor can reassign its parameters (here, `value`) to normalize input; the reassigned values are what get stored in the record's fields, since field assignment is implicit and happens after the compact constructor body runs.

## Defensive-Copying Mutable Components

```java
public record Team(String name, List<String> members) {
    public Team {
        Objects.requireNonNull(name, "name must not be null");
        members = List.copyOf(members); // both validates non-null and prevents external mutation
    }
}
```

## See Also

- [`api-record-data-carrier`](api-record-data-carrier.md) - Why records are the right tool for immutable data before adding validation
- [`api-defensive-copy-mutable-args`](api-defensive-copy-mutable-args.md) - The general defensive-copy principle applied here to record components
- [`err-fail-fast-validation`](err-fail-fast-validation.md) - Failing fast at the construction boundary rather than downstream
- [`null-requireNonNull-guard`](null-requireNonNull-guard.md) - Guarding record components against null in the compact constructor
