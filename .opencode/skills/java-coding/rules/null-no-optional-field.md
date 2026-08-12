# null-no-optional-field

> Never declare a field of type `Optional<T>`

## Why It Matters

`Optional` was designed as a return-type wrapper, not a general-purpose "maybe" container, and it does not implement `Serializable`. Storing it as a field bloats every instance with an extra wrapper object, breaks serialization frameworks (Jackson, JPA, `java.io.Serializable`), and pushes the null-check burden onto every field accessor instead of solving it once.

## Bad

```java
public class Customer {

    private final String name;
    private final Optional<String> middleName; // extra allocation per instance

    public Customer(String name, Optional<String> middleName) {
        this.name = name;
        this.middleName = middleName;
    }

    public Optional<String> getMiddleName() {
        return middleName;
    }
}

// JPA/Jackson will not serialize this cleanly - Optional is not Serializable
Customer customer = new Customer("Ada", Optional.of("Lovelace"));
```

## Good

```java
public class Customer {

    private final String name;
    private final String middleName; // nullable, plain field

    public Customer(String name, String middleName) {
        this.name = Objects.requireNonNull(name, "name");
        this.middleName = middleName; // may legitimately be null
    }

    // Expose the "maybe" only at the API boundary, not in storage
    public Optional<String> middleName() {
        return Optional.ofNullable(middleName);
    }
}
```

## Records Follow The Same Rule

```java
// Bad - Optional in a record component
public record Address(String street, Optional<String> unit) {}

// Good - nullable component, Optional exposed via accessor method if needed
public record Address(String street, String unit) {
    public Optional<String> unitNumber() {
        return Optional.ofNullable(unit);
    }
}
```

## See Also

- [`null-optional-return-type`](null-optional-return-type.md) - Use `Optional<T>` for return types only
- [`null-no-optional-param`](null-no-optional-param.md) - Never accept `Optional<T>` as a method parameter
- [`null-nullable-annotation`](null-nullable-annotation.md) - Annotate nullable fields explicitly
- [`api-record-data-carrier`](api-record-data-carrier.md) - Use records for plain data carriers
