# name-classes-pascal

> Use `PascalCase` for classes, interfaces, enums, records

## Why It Matters

Java's naming conventions are load-bearing: tooling, IDEs, and every Java developer's muscle memory assume `PascalCase` identifies a type and `camelCase` identifies a variable or method. Breaking this convention forces readers to stop and parse context instead of pattern-matching on shape, and it clashes with generated code, reflection-based frameworks, and style checkers (Checkstyle, Google Java Format) that assume it.

## Bad

```java
public class invoiceProcessor {  // lowercase start looks like a variable
    // ...
}

interface data_repository {  // snake_case is not idiomatic Java
    void save(Object entity);
}

enum orderStatus {  // should read as a type name
    PENDING, SHIPPED, DELIVERED
}

record userProfile(String name, int age) {  // records are types too
}
```

## Good

```java
public class InvoiceProcessor {
    // ...
}

interface DataRepository {
    void save(Object entity);
}

enum OrderStatus {
    PENDING, SHIPPED, DELIVERED
}

record UserProfile(String name, int age) {
}
```

## Nested and Inner Types

```java
public class HttpClient {

    // Nested types remain PascalCase, regardless of static/inner status
    public static class Builder {
        private String baseUrl;

        public Builder baseUrl(String baseUrl) {
            this.baseUrl = baseUrl;
            return this;
        }
    }

    private enum ConnectionState {
        IDLE, CONNECTING, CONNECTED, CLOSED
    }
}
```

Annotation types follow the same rule: `@interface Retryable`, not `@interface retryable`.

## See Also

- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
- [`name-constants-screaming-snake`](name-constants-screaming-snake.md) - Use SCREAMING_SNAKE_CASE for constants
- [`name-acronyms-as-words`](name-acronyms-as-words.md) - Treat acronyms as words in identifiers
- [`name-packages-lowercase`](name-packages-lowercase.md) - Use all-lowercase package names
