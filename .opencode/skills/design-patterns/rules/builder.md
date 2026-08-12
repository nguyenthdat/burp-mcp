# builder

> Construct a complex value through optional or ordered steps, validating before producing the final immutable result.

## Intent & Pressure

Reach for Builder when a type has many optional fields, construction has ordering constraints, validation must happen once at the end, or one construction process can yield distinct product representations. The pressure is combinatorial: a constructor with a dozen optional parameters (or a dozen overloads) becomes unreadable and error-prone, and default-then-mutate leaves callers with objects that can be observed in an invalid state.

Do not reach for it when a type has two or three required fields — a plain constructor or named/keyword arguments are simpler and just as safe. Fluent setters alone are not necessarily Builder; the defining trait is validated, one-shot construction of an otherwise-immutable value.

## Native-Construct Alternative

Named/keyword constructor arguments (Python, Kotlin, C#, Swift `init` with defaults) already solve the "too many parameters" problem without a separate type. Reach for a real Builder only when steps are genuinely ordered, optional in combination (some combinations are invalid), or when validation must be deferred to a single `build()`/`finish()` call.

## Language Implementations

### Rust

```rust
#[derive(Default)]
pub struct RequestBuilder {
    url: Option<String>,
    method: Method,
    headers: Vec<(String, String)>,
}

impl RequestBuilder {
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
    #[must_use]
    pub fn build(self) -> Result<Request, BuildError> {
        let url = self.url.ok_or(BuildError::MissingUrl)?;
        Ok(Request { url, method: self.method, headers: self.headers })
    }
}
```

A consuming builder (`self`, not `&mut self`) plus `#[must_use]` on `build` prevents both partial reuse bugs and silently discarded builders.

### TypeScript

```typescript
class RequestBuilder {
  private url?: string;
  private headers: [string, string][] = [];

  setUrl(url: string): this {
    this.url = url;
    return this;
  }
  addHeader(key: string, value: string): this {
    this.headers.push([key, value]);
    return this;
  }
  build(): Request {
    if (!this.url) throw new Error("url is required");
    return { url: this.url, headers: this.headers };
  }
}
```

### Python

```python
from dataclasses import dataclass, field

@dataclass
class RequestBuilder:
    _url: str | None = None
    _headers: list[tuple[str, str]] = field(default_factory=list)

    def url(self, url: str) -> "RequestBuilder":
        self._url = url
        return self

    def header(self, key: str, value: str) -> "RequestBuilder":
        self._headers.append((key, value))
        return self

    def build(self) -> "Request":
        if self._url is None:
            raise ValueError("url is required")
        return Request(url=self._url, headers=list(self._headers))
```

Python often prefers keyword-argument constructors with defaults over a Builder class; reach for the class above only when steps are ordered or validation spans multiple fields.

### Go

```go
type RequestBuilder struct {
    url     string
    headers [][2]string
}

func NewRequestBuilder() *RequestBuilder { return &RequestBuilder{} }

func (b *RequestBuilder) URL(url string) *RequestBuilder {
    b.url = url
    return b
}
func (b *RequestBuilder) Header(key, value string) *RequestBuilder {
    b.headers = append(b.headers, [2]string{key, value})
    return b
}
func (b *RequestBuilder) Build() (Request, error) {
    if b.url == "" {
        return Request{}, errors.New("url is required")
    }
    return Request{URL: b.url, Headers: b.headers}, nil
}
```

Go more commonly uses "functional options" (`NewRequest(url, WithHeader(k, v))`) for optional construction; use a fluent builder when steps are ordered or the option list is large and stateful.

### C#

```csharp
public sealed class RequestBuilder
{
    private string? _url;
    private readonly List<(string Key, string Value)> _headers = new();

    public RequestBuilder WithUrl(string url) { _url = url; return this; }
    public RequestBuilder WithHeader(string key, string value)
    {
        _headers.Add((key, value));
        return this;
    }

    public Request Build()
    {
        if (_url is null) throw new InvalidOperationException("url is required");
        return new Request(_url, _headers.ToArray());
    }
}
```

C# records with `with`-expressions or required init-only properties often replace Builder for simple immutable data; keep Builder for multi-step, validated construction.

### Kotlin

```kotlin
class RequestBuilder {
    private var url: String? = null
    private val headers = mutableListOf<Pair<String, String>>()

    fun url(url: String) = apply { this.url = url }
    fun header(key: String, value: String) = apply { headers.add(key to value) }

    fun build(): Request {
        val url = checkNotNull(url) { "url is required" }
        return Request(url, headers.toList())
    }
}

// usage with a DSL-style builder lambda
val request = RequestBuilder().apply {
    url("https://example.com")
    header("Accept", "application/json")
}.build()
```

Kotlin frequently prefers a lambda-with-receiver DSL builder (as above) over Java-style chained setters.

### C

```c
typedef struct request_builder {
    char *url;
    header_t *headers;
    size_t header_count;
} request_builder_t;

void request_builder_init(request_builder_t *b) { memset(b, 0, sizeof(*b)); }
void request_builder_set_url(request_builder_t *b, const char *url) {
    b->url = strdup(url);
}
int request_builder_add_header(request_builder_t *b, const char *k, const char *v) { /* ... */ return 0; }

/* returns 0 on success; caller owns *out and must call request_destroy */
int request_builder_build(request_builder_t *b, request_t *out) {
    if (!b->url) return -1;
    out->url = b->url;
    out->headers = b->headers;
    out->header_count = b->header_count;
    return 0;
}
```

### C++

```cpp
class RequestBuilder {
public:
    RequestBuilder &url(std::string url) { url_ = std::move(url); return *this; }
    RequestBuilder &header(std::string key, std::string value) {
        headers_.emplace_back(std::move(key), std::move(value));
        return *this;
    }
    [[nodiscard]] Request build() && {
        if (url_.empty()) throw std::invalid_argument("url is required");
        return Request{std::move(url_), std::move(headers_)};
    }
private:
    std::string url_;
    std::vector<std::pair<std::string, std::string>> headers_;
};
```

The ref-qualified `build() &&` only allows calling `build` on an rvalue (a builder about to be discarded), preventing accidental reuse of a partially-consumed builder.

### Swift

```swift
struct RequestBuilder {
    private var url: String?
    private var headers: [(String, String)] = []

    func url(_ url: String) -> RequestBuilder {
        var copy = self
        copy.url = url
        return copy
    }
    func header(_ key: String, _ value: String) -> RequestBuilder {
        var copy = self
        copy.headers.append((key, value))
        return copy
    }
    func build() throws -> Request {
        guard let url else { throw BuildError.missingUrl }
        return Request(url: url, headers: headers)
    }
}
```

Swift's value-type `struct` builder returns copies instead of mutating in place, so intermediate builder values stay independent — often preferable to a reference-type fluent builder.

## Pitfalls

- Silently defaulting a missing required field instead of failing `build()`.
- Making the builder mutable and shared across threads/goroutines without documenting that it isn't safe for concurrent use.
- Fluent setters that mutate and return `this` while also being reused after `build()`, causing surprising aliasing.
- Building a full Builder class for two or three required fields, when named arguments would do.
- Forgetting `#[must_use]`/`[[nodiscard]]`/linter checks, so a constructed-but-undiscarded builder or result hides a bug.

## See Also

- [factory-method](factory-method.md) — simpler single-call construction when there's no multi-step process.
- [abstract-factory](abstract-factory.md) — when whole families of built products must stay compatible.
- [prototype](prototype.md) — copying an existing built value instead of re-running the builder.
