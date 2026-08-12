# name-boolean-is-has-can

> Name boolean accessors `isX`/`hasX`/`canX`

## Why It Matters

A boolean method or field named as a question (`isValid`, `hasPermission`, `canRetry`) reads naturally at call sites (`if (order.isPaid())`) and signals its type without needing to check the signature. Bean-aware frameworks (Jackson, JavaBeans introspection, JSP EL) also specifically recognize the `is` prefix for `boolean`-typed properties, so deviating from it can silently break serialization or property discovery.

## Bad

```java
public class Order {

    private boolean paid;
    private boolean urgent;

    public boolean paid() {   // ambiguous: getter or verb?
        return paid;
    }

    public boolean getUrgent() {  // "get" on a boolean is non-idiomatic
        return urgent;
    }

    public boolean permission(User user) {  // reads like a noun, not a predicate
        return user.hasRole("ADMIN");
    }
}
```

## Good

```java
public class Order {

    private boolean paid;
    private boolean urgent;

    public boolean isPaid() {
        return paid;
    }

    public boolean isUrgent() {
        return urgent;
    }

    public boolean hasPermission(User user) {
        return user.hasRole("ADMIN");
    }

    public boolean canCancel() {
        return !paid && !urgent;
    }
}
```

## Boxed `Boolean` and Records

For a boxed `Boolean` field, `is`-prefixed getters remain the convention in most codebases, though some style guides drop the prefix to signal nullability explicitly - be consistent within a project either way.

```java
public class FeatureFlag {
    private Boolean enabled;  // nullable: unknown vs. true vs. false

    public Boolean isEnabled() {  // still conventional despite boxed type
        return enabled;
    }
}
```

Records generate accessors matching the component name exactly, without an `is` prefix, since that is fixed by the language - so prefer naming the component itself as a predicate where it improves readability at the call site:

```java
public record AccessCheck(boolean granted) {
}

AccessCheck check = new AccessCheck(true);
if (check.granted()) {  // no isGranted() - records don't add "is"
    // ...
}
```

## See Also

- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
- [`name-getter-setter-bean-convention`](name-getter-setter-bean-convention.md) - Follow JavaBean getter/setter naming for accessors
- [`api-record-data-carrier`](api-record-data-carrier.md) - Use records for simple data carriers
