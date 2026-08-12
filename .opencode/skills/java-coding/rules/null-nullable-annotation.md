# null-nullable-annotation

> Annotate nullability with `@Nullable`/`@NonNull` (JSpecify)

## Why It Matters

Without explicit annotations, "can this be null?" is a question every reader has to answer by tracing call chains or reading (often absent) documentation. Nullability annotations make the contract part of the signature itself, so IDEs and static analyzers (NullAway, Error Prone, IntelliJ) can catch violations at compile or build time instead of at 3 a.m. in production.

## Bad

```java
public class UserService {

    // Is the return value ever null? Can name be null? Nobody knows without reading the body.
    public User findByName(String name) {
        if (name == null) {
            return null;
        }
        return repository.lookup(name);
    }
}

// Caller has no signal and guesses wrong
User user = service.findByName(input);
user.getEmail().toLowerCase(); // NPE risk hidden by the signature
```

## Good

```java
import org.jspecify.annotations.Nullable;

public class UserService {

    public @Nullable User findByName(String name) {
        // 'name' has no annotation - under @NullMarked it is implicitly non-null
        return repository.lookup(name);
    }
}

// The signature itself documents the contract
@Nullable User user = service.findByName(input);
if (user != null) {
    user.getEmail().toLowerCase();
}
```

## Combine With Static Analysis

```java
// build.gradle / pom.xml: add JSpecify + NullAway as a compiler plugin
// NullAway then flags this at build time, not at runtime:
public String greet(@Nullable String name) {
    return "Hello, " + name.toUpperCase(); // NullAway error: dereference of @Nullable value
}
```

## Package-Level Default

Prefer marking whole packages `@NullMarked` (see `null-jspecify-nullmarked`) so `@Nullable` is the only annotation you ever need to write, and every unannotated type defaults to non-null.

## See Also

- [`null-jspecify-nullmarked`](null-jspecify-nullmarked.md) - Adopt `@NullMarked` at the package level
- [`lint-nullaway-annotation-checking`](lint-nullaway-annotation-checking.md) - Enforce nullability with NullAway in CI
- [`null-requireNonNull-guard`](null-requireNonNull-guard.md) - Guard constructors/methods with `Objects.requireNonNull`
- [`doc-javadoc-param-return-throws`](doc-javadoc-param-return-throws.md) - Document parameters, returns, and thrown exceptions
