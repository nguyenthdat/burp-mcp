# null-optional-return-type

> Use `Optional<T>` for return types, never for fields or parameters

## Why It Matters

`Optional<T>` exists to make "this method might not have an answer" visible in the method signature, forcing callers to handle absence explicitly instead of risking a `NullPointerException` three call frames away. Its design intent is strictly limited to return values from methods that logically may not produce a result — using it elsewhere (fields, parameters, collection elements) adds overhead and confusion without the compile-time safety benefit.

## Bad

```java
public class UserRepository {

    public User findById(String id) {
        // Returns null when not found - caller has no signal to check
        return database.query(id);
    }

    public User findActiveByEmail(String email) {
        User user = database.queryByEmail(email);
        if (user == null || !user.isActive()) {
            return null; // silent, easy to forget to check
        }
        return user;
    }
}

// Caller code that will eventually NPE
User user = repository.findById("123");
System.out.println(user.getName()); // boom if not found
```

## Good

```java
public class UserRepository {

    public Optional<User> findById(String id) {
        return Optional.ofNullable(database.query(id));
    }

    public Optional<User> findActiveByEmail(String email) {
        return Optional.ofNullable(database.queryByEmail(email))
                .filter(User::isActive);
    }
}

// Caller is forced to confront the empty case
String name = repository.findById("123")
        .map(User::getName)
        .orElse("Unknown");
```

## When Not To Use Optional

`Optional` is for a single, possibly-absent return value. It is the wrong tool for:

- Fields (`null-no-optional-field`) — not serializable, adds indirection to every access.
- Method parameters (`null-no-optional-param`) — overloading or a builder communicates optionality better.
- Collections — an empty `List<T>` already expresses "no results" (`null-empty-collection-not-null`).
- Primitive results — prefer `OptionalInt`/`OptionalLong`/`OptionalDouble` (`null-optional-primitive`).

## See Also

- [`null-no-optional-field`](null-no-optional-field.md) - Never declare a field of type `Optional<T>`
- [`null-no-optional-param`](null-no-optional-param.md) - Never accept `Optional<T>` as a method parameter
- [`null-optional-chaining`](null-optional-chaining.md) - Chain `Optional` with functional methods
- [`anti-return-null-instead-optional`](anti-return-null-instead-optional.md) - Returning `null` instead of `Optional`
