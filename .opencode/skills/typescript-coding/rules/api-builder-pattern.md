# api-builder-pattern

> Use a builder/fluent API for objects with many optional construction parameters

## Why It Matters

A constructor or factory function with many optional parameters forces callers to either remember positional order (error-prone and unreadable at the call site) or pass a large options object where required fields are indistinguishable from optional ones. A fluent builder makes each configuration step self-documenting, allows validation to happen once at `.build()` instead of scattered through the constructor, and — when combined with TypeScript's type system — can enforce at compile time that required fields were actually set before building.

## Bad

```typescript
class HttpClient {
  constructor(
    baseUrl: string,
    timeoutMs: number,
    retries: number,
    authToken?: string,
    userAgent?: string,
  ) {}
}

// Which argument is which? Easy to swap timeoutMs and retries by accident.
const client = new HttpClient("https://api.example.com", 30000, 3, undefined, "MyApp/1.0");
```

## Good

```typescript
class HttpClientBuilder {
  private timeoutMs = 30_000;
  private retries = 3;
  private authToken?: string;
  private userAgent?: string;

  constructor(private readonly baseUrl: string) {}

  withTimeout(ms: number): this {
    this.timeoutMs = ms;
    return this;
  }

  withRetries(n: number): this {
    this.retries = n;
    return this;
  }

  withAuthToken(token: string): this {
    this.authToken = token;
    return this;
  }

  withUserAgent(ua: string): this {
    this.userAgent = ua;
    return this;
  }

  build(): HttpClient {
    return new HttpClient(this.baseUrl, this.timeoutMs, this.retries, this.authToken, this.userAgent);
  }
}

const client = new HttpClientBuilder("https://api.example.com")
  .withTimeout(10_000)
  .withRetries(5)
  .withAuthToken("secret")
  .build();
```

## Compile-Time Required-Field Enforcement (Typestate Builder)

```typescript
type NoUrl = { baseUrl?: never };
type HasUrl = { baseUrl: string };

class TypedBuilder<State extends { baseUrl?: string } = NoUrl> {
  private config: Partial<HttpClientConfig> = {};

  baseUrl(this: TypedBuilder<NoUrl>, url: string): TypedBuilder<HasUrl> {
    this.config.baseUrl = url;
    return this as unknown as TypedBuilder<HasUrl>;
  }

  build(this: TypedBuilder<HasUrl>): HttpClient {
    return new HttpClient(this.config as HttpClientConfig);
  }
}

// new TypedBuilder().build(); // compile error: build() requires HasUrl state
const client = new TypedBuilder().baseUrl("https://api.example.com").build(); // OK
```

## When a Plain Options Object Is Enough

For fewer than roughly 4-5 optional fields with no ordering or validation dependency between them, a single options object (`function createClient(opts: ClientOptions)`) is simpler than a builder and just as readable — reserve builders for genuinely complex, multi-step, or conditionally-required construction.

## See Also

- [api-generic-defaults](api-generic-defaults.md) - Give generic type parameters sensible defaults where one exists
- [api-avoid-optional-overuse](api-avoid-optional-overuse.md) - Avoid excessive optional properties; model valid states as required unions instead
- [async-no-async-constructor](async-no-async-constructor.md) - Avoid `async` constructors; use a static async factory method instead
