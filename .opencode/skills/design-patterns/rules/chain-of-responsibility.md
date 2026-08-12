# chain-of-responsibility

> Pass a request through an ordered sequence of handlers, any of which may handle, transform, reject, or forward it.

## Intent & Pressure

Reach for Chain of Responsibility when a request must be evaluated by an ordered set of handlers where any one might fully handle it (stopping the chain), veto it, or pass it along — validation pipelines, middleware stacks, event-bubbling UI systems, support-ticket escalation. The pressure is a variable, potentially reorderable set of handlers where the caller shouldn't need to know which one will actually respond.

Do not reach for it when the handler sequence is fixed and short — a single function with early returns/`if`/`elif` is simpler and just as readable. The chain earns its complexity when handlers are added/removed/reordered independently (e.g., via plugins or configuration).

## Native-Construct Alternative

A single function with sequential early returns handles the common fixed-sequence case. Promote to a real chain when handlers must be independently pluggable, come from different modules/plugins, or need uniform continue/handled/rejected semantics enforced by a shared contract.

## Language Implementations

### Rust

```rust
enum Outcome { Handled(Response), NotHandled }

trait Handler {
    fn handle(&self, req: &Request) -> Outcome;
}

struct Chain {
    handlers: Vec<Box<dyn Handler>>,
}

impl Chain {
    fn dispatch(&self, req: &Request) -> Option<Response> {
        for handler in &self.handlers {
            if let Outcome::Handled(response) = handler.handle(req) {
                return Some(response);
            }
        }
        None
    }
}
```

An explicit `Outcome` enum (not a boolean) makes "handled vs. not handled" unambiguous and lets a future variant add "rejected" without breaking exhaustiveness elsewhere.

### TypeScript

```typescript
type Outcome = { handled: true; response: Response } | { handled: false };

interface Handler {
  handle(req: Request): Outcome;
}

class Chain {
  constructor(private handlers: Handler[]) {}
  dispatch(req: Request): Response | undefined {
    for (const handler of this.handlers) {
      const outcome = handler.handle(req);
      if (outcome.handled) return outcome.response;
    }
    return undefined;
  }
}
```

### Python

```python
from typing import Protocol

class Handler(Protocol):
    def handle(self, request: Request) -> Response | None: ...

class Chain:
    def __init__(self, handlers: list[Handler]) -> None:
        self._handlers = handlers

    def dispatch(self, request: Request) -> Response | None:
        for handler in self._handlers:
            response = handler.handle(request)
            if response is not None:
                return response
        return None
```

### Go

```go
type Handler interface {
    Handle(req Request) (Response, bool)
}

type Chain struct {
    handlers []Handler
}

func (c Chain) Dispatch(req Request) (Response, bool) {
    for _, h := range c.handlers {
        if resp, ok := h.Handle(req); ok {
            return resp, true
        }
    }
    return Response{}, false
}
```

### C#

```csharp
public interface IHandler
{
    bool TryHandle(Request request, out Response response);
}

public sealed class Chain
{
    private readonly IReadOnlyList<IHandler> _handlers;
    public Chain(IReadOnlyList<IHandler> handlers) => _handlers = handlers;

    public Response? Dispatch(Request request)
    {
        foreach (var handler in _handlers)
        {
            if (handler.TryHandle(request, out var response)) return response;
        }
        return null;
    }
}
```

### Kotlin

```kotlin
fun interface Handler {
    fun handle(request: Request): Response?
}

class Chain(private val handlers: List<Handler>) {
    fun dispatch(request: Request): Response? =
        handlers.firstNotNullOfOrNull { it.handle(request) }
}
```

### C

```c
typedef struct handler {
    int (*handle)(struct handler *self, const request_t *req, response_t *out);
} handler_t;

int chain_dispatch(handler_t **handlers, size_t count, const request_t *req, response_t *out) {
    for (size_t i = 0; i < count; i++) {
        if (handlers[i]->handle(handlers[i], req, out) == 0) {
            return 0; /* handled */
        }
    }
    return -1; /* not handled */
}
```

### C++

```cpp
class Handler {
public:
    virtual ~Handler() = default;
    virtual std::optional<Response> handle(const Request &req) = 0;
};

class Chain {
public:
    explicit Chain(std::vector<std::unique_ptr<Handler>> handlers) : handlers_(std::move(handlers)) {}

    std::optional<Response> dispatch(const Request &req) const {
        for (const auto &handler : handlers_) {
            if (auto response = handler->handle(req)) return response;
        }
        return std::nullopt;
    }
private:
    std::vector<std::unique_ptr<Handler>> handlers_;
};
```

### Swift

```swift
protocol Handler {
    func handle(_ request: Request) -> Response?
}

struct Chain {
    let handlers: [Handler]
    func dispatch(_ request: Request) -> Response? {
        for handler in handlers {
            if let response = handler.handle(request) { return response }
        }
        return nil
    }
}
```

## Pitfalls

- Ambiguous "did anyone handle this?" signal (e.g., `null`/empty string doubling as both "not handled" and "handled with nothing") instead of an explicit outcome type.
- A handler that continues mutating shared request state after another handler already claimed to have handled it.
- Order-dependent handlers with no documented or enforced ordering, so adding a new handler silently changes behavior.
- Unbounded chain length with no depth/timeout guard when handlers can recurse or dispatch sub-requests.
- Swallowing a handler's error instead of propagating a typed failure distinctly from "not handled."

## See Also

- [command](command.md) — encapsulating the request itself as an object, often passed through a chain.
- [pipeline-middleware](pipeline-middleware.md) — the generalized, request/response-transforming version of this pattern.
- [strategy](strategy.md) — choosing one algorithm, versus trying several in sequence until one applies.
