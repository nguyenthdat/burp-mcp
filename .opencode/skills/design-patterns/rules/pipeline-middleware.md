# pipeline-middleware

> Compose an ordered set of cross-cutting request/response processing stages, each able to act before and/or after calling the next.

## Intent & Pressure

Reach for Pipeline/Middleware when requests or messages must pass through a configurable, ordered set of cross-cutting steps (auth, logging, rate limiting, compression, tracing) that should be composable and independently addable/removable, and each step needs the ability to short-circuit or wrap the call to the next stage. The pressure is the generalized, request/response-aware version of [decorator](decorator.md): most web frameworks, RPC clients, and message processors already model this.

Do not reach for a custom pipeline abstraction when the framework you're using already provides one (ASP.NET Core middleware, Express middleware, Tower/Axum layers, Go's `http.Handler` wrapping) — use that instead of inventing a parallel mechanism. Build a bespoke pipeline only for non-HTTP request/response flows the framework doesn't cover (e.g., a message-queue consumer pipeline).

## Native-Construct Alternative

For a small, fixed set of cross-cutting steps, plain nested function calls (`logged(rateLimited(handler))`) achieve the same effect without a pipeline abstraction. Reach for a real middleware/pipeline type when the step set is configured dynamically (from a list, from plugins) rather than hard-coded in source.

## Language Implementations

### Rust

```rust
type Next<'a> = Box<dyn Fn(Request) -> Result<Response, Error> + 'a>;
type Middleware = Box<dyn Fn(Request, Next) -> Result<Response, Error>>;

fn build_pipeline(middlewares: Vec<Middleware>, handler: impl Fn(Request) -> Result<Response, Error> + 'static) -> impl Fn(Request) -> Result<Response, Error> {
    middlewares.into_iter().rev().fold(Box::new(handler) as Next, |next, mw| {
        Box::new(move |req| mw(req, next))
    })
}
```

Frameworks like `axum`/`tower` express this as `Layer`/`Service` composition; prefer the framework's own middleware trait over a bespoke one when using it.

### TypeScript

```typescript
type Next = (req: Request) => Promise<Response>;
type Middleware = (req: Request, next: Next) => Promise<Response>;

function compose(middlewares: Middleware[], handler: Next): Next {
  return middlewares.reduceRight(
    (next, mw) => (req) => mw(req, next),
    handler,
  );
}

const loggingMiddleware: Middleware = async (req, next) => {
  console.log(`-> ${req.path}`);
  const response = await next(req);
  console.log(`<- ${response.status}`);
  return response;
};
```

### Python

```python
from typing import Callable, Awaitable

Handler = Callable[[Request], Awaitable[Response]]
Middleware = Callable[[Request, Handler], Awaitable[Response]]

def compose(middlewares: list[Middleware], handler: Handler) -> Handler:
    for mw in reversed(middlewares):
        handler = (lambda h, mw=mw: (lambda req: mw(req, h)))(handler)
    return handler
```

ASGI frameworks (Starlette/FastAPI) already provide this as `app.middleware("http")` decorators — prefer that over a hand-rolled composer in production code.

### Go

```go
type Middleware func(http.Handler) http.Handler

func Logging(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        log.Printf("-> %s", r.URL.Path)
        next.ServeHTTP(w, r)
    })
}

func Chain(h http.Handler, middlewares ...Middleware) http.Handler {
    for i := len(middlewares) - 1; i >= 0; i-- {
        h = middlewares[i](h)
    }
    return h
}
```

Wrapping `http.Handler` is the idiomatic Go middleware pattern — the standard library's own interface is the pipeline contract.

### C#

```csharp
public delegate Task<HttpResponse> RequestDelegate(HttpRequest req);

public static RequestDelegate Compose(IEnumerable<Func<RequestDelegate, RequestDelegate>> middlewares, RequestDelegate handler)
{
    foreach (var middleware in middlewares.Reverse())
    {
        handler = middleware(handler);
    }
    return handler;
}
```

ASP.NET Core's own `IApplicationBuilder.Use(...)` pipeline is this exact pattern already built in — use it directly instead of a custom composer for HTTP.

### Kotlin

```kotlin
typealias Handler = suspend (Request) -> Response
typealias Middleware = suspend (Request, Handler) -> Response

fun compose(middlewares: List<Middleware>, handler: Handler): Handler =
    middlewares.foldRight(handler) { middleware, next ->
        { req -> middleware(req, next) }
    }
```

Ktor's own `Plugin`/interceptor pipeline already implements this; use it for HTTP servers instead of a custom composer.

### C

```c
typedef struct request request_t;
typedef struct response response_t;
typedef response_t (*handler_fn)(const request_t *req, void *ctx);
typedef response_t (*middleware_fn)(const request_t *req, handler_fn next, void *next_ctx, void *mw_ctx);

response_t logging_middleware(const request_t *req, handler_fn next, void *next_ctx, void *mw_ctx) {
    log_request(req);
    response_t resp = next(req, next_ctx);
    log_response(&resp);
    return resp;
}
```

C middleware is typically composed by hand at startup (building a fixed call chain) rather than dynamically, since there's no closure capture without manually threading context pointers.

### C++

```cpp
using Handler = std::function<Response(const Request &)>;
using Middleware = std::function<Response(const Request &, const Handler &)>;

Handler compose(std::vector<Middleware> middlewares, Handler handler) {
    for (auto it = middlewares.rbegin(); it != middlewares.rend(); ++it) {
        Middleware mw = *it;
        Handler next = handler;
        handler = [mw, next](const Request &req) { return mw(req, next); };
    }
    return handler;
}
```

### Swift

```swift
typealias Handler = (Request) async throws -> Response
typealias Middleware = (Request, Handler) async throws -> Response

func compose(_ middlewares: [Middleware], handler: @escaping Handler) -> Handler {
    middlewares.reversed().reduce(handler) { next, middleware in
        { req in try await middleware(req, next) }
    }
}
```

Vapor's own middleware pipeline (`app.middleware.use(...)`) already implements this for HTTP servers.

## Pitfalls

- Reinventing a pipeline abstraction the web/RPC framework already provides, instead of using its native middleware mechanism.
- A middleware that forgets to call `next`, silently short-circuiting the pipeline without the caller realizing it.
- Order-sensitive middleware (auth before rate-limiting, or vice versa) with no documented or enforced ordering contract.
- Middleware that mutates shared request/response state without considering concurrent requests reusing the same pipeline instance.
- Swallowing an inner stage's error instead of propagating or explicitly translating it, hiding failures from outer middleware that need to react to them.

## See Also

- [decorator](decorator.md) — the underlying single-object wrapping mechanism this pattern generalizes to request/response flows.
- [chain-of-responsibility](chain-of-responsibility.md) — similar ordered-handler shape, but typically "first handler that claims it" rather than "every stage runs."
- [circuit-breaker](circuit-breaker.md) — a common middleware stage protecting a specific downstream call.
