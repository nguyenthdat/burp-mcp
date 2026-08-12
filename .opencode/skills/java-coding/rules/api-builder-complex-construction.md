# api-builder-complex-construction

> Use the Builder pattern for complex construction

## Why It Matters

When a type has many optional fields, constructors force callers into positional parameter lists that are error-prone and unreadable at the call site. A builder makes each value's meaning explicit by name, lets you validate the fully-assembled object in one place, and scales gracefully as new optional fields are added without breaking every existing call site.

## Bad

```java
public class HttpRequest {
    private final String url;
    private final String method;
    private final Map<String, String> headers;
    private final byte[] body;
    private final int timeoutMs;
    private final boolean followRedirects;
    private final boolean retryOnFailure;

    // Which boolean is which? Callers must count positions.
    public HttpRequest(String url, String method, Map<String, String> headers,
                        byte[] body, int timeoutMs, boolean followRedirects,
                        boolean retryOnFailure) {
        this.url = url;
        this.method = method;
        this.headers = headers;
        this.body = body;
        this.timeoutMs = timeoutMs;
        this.followRedirects = followRedirects;
        this.retryOnFailure = retryOnFailure;
    }
}

// Call site is unreadable and fragile to reordering
HttpRequest request = new HttpRequest(
        "https://api.example.com", "POST", Map.of(), null, 5000, true, false);
```

## Good

```java
public final class HttpRequest {
    private final String url;
    private final String method;
    private final Map<String, String> headers;
    private final byte[] body;
    private final int timeoutMs;
    private final boolean followRedirects;
    private final boolean retryOnFailure;

    private HttpRequest(Builder builder) {
        this.url = Objects.requireNonNull(builder.url, "url");
        this.method = builder.method;
        this.headers = Map.copyOf(builder.headers);
        this.body = builder.body;
        this.timeoutMs = builder.timeoutMs;
        this.followRedirects = builder.followRedirects;
        this.retryOnFailure = builder.retryOnFailure;
    }

    public static Builder builder(String url) {
        return new Builder(url);
    }

    public static final class Builder {
        private final String url;
        private String method = "GET";
        private final Map<String, String> headers = new HashMap<>();
        private byte[] body;
        private int timeoutMs = 10_000;
        private boolean followRedirects = true;
        private boolean retryOnFailure = false;

        private Builder(String url) {
            this.url = url;
        }

        public Builder method(String method) {
            this.method = method;
            return this;
        }

        public Builder header(String name, String value) {
            this.headers.put(name, value);
            return this;
        }

        public Builder body(byte[] body) {
            this.body = body;
            return this;
        }

        public Builder timeout(Duration timeout) {
            this.timeoutMs = Math.toIntExact(timeout.toMillis());
            return this;
        }

        public Builder retryOnFailure(boolean retry) {
            this.retryOnFailure = retry;
            return this;
        }

        public HttpRequest build() {
            return new HttpRequest(this);
        }
    }
}

// Call site is self-documenting
HttpRequest request = HttpRequest.builder("https://api.example.com")
        .method("POST")
        .header("Content-Type", "application/json")
        .timeout(Duration.ofSeconds(5))
        .build();
```

## Combining With Records

For simpler cases where every field is required and the object is small, prefer a record with a static factory over a builder — builders earn their keep when there are several optional fields or when construction needs staged validation.

## See Also

- [`api-avoid-telescoping-constructors`](api-avoid-telescoping-constructors.md) - The problem builders solve
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - Lighter-weight alternative for simple construction
- [`api-fluent-method-chaining`](api-fluent-method-chaining.md) - Designing the chained methods a builder exposes
- [`api-defensive-copy-mutable-args`](api-defensive-copy-mutable-args.md) - Copying mutable builder state before storing it
