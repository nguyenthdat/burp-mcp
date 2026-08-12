# factory-method

> Defer creation of one product to a subclass, implementation, or function so callers depend on an abstraction, not a concrete type.

## Intent & Pressure

Reach for Factory Method when a shared workflow must create an object whose concrete type depends on configuration, environment, or a subtype that isn't known until later — and new creation policies are expected to arrive over time (a new file format, a new payment provider, a new notification channel). The pressure is: callers should not need to change when a new concrete product is added; only the factory needs to know about it.

Do not reach for it when there is exactly one concrete product today and no credible second one on the roadmap — a constructor or a plain function is simpler and just as clear. Do not use it merely to "future proof" a type with a single implementation; that adds an indirection with no payoff and slows down readers.

## Native-Construct Alternative

Most languages have a direct constructor idiom that is sufficient until real variation shows up: `new`/`TryFrom`/associated function (Rust), a plain constructor or a static `create` function (TypeScript/Python/Kotlin/C#), a package-level constructor function returning an interface (Go), or a `_create` function returning an opaque handle (C). Start there. Promote to Factory Method only once a second creation policy is real and the call sites must stay decoupled from it.

## Language Implementations

### Rust

```rust
trait Notifier {
    fn send(&self, message: &str) -> Result<(), NotifyError>;
}

struct EmailNotifier;
struct SmsNotifier;

impl Notifier for EmailNotifier {
    fn send(&self, message: &str) -> Result<(), NotifyError> { /* ... */ Ok(()) }
}
impl Notifier for SmsNotifier {
    fn send(&self, message: &str) -> Result<(), NotifyError> { /* ... */ Ok(()) }
}

fn make_notifier(kind: NotifierKind) -> Box<dyn Notifier> {
    match kind {
        NotifierKind::Email => Box::new(EmailNotifier),
        NotifierKind::Sms => Box::new(SmsNotifier),
    }
}
```

Prefer an `enum` + `match` factory function returning `Box<dyn Trait>` only when call sites must store a runtime-selected implementation; a generic constructor keeps static call sites monomorphized.

### TypeScript

```typescript
interface Notifier {
  send(message: string): Promise<void>;
}

class EmailNotifier implements Notifier {
  async send(message: string): Promise<void> { /* ... */ }
}
class SmsNotifier implements Notifier {
  async send(message: string): Promise<void> { /* ... */ }
}

function createNotifier(kind: "email" | "sms"): Notifier {
  switch (kind) {
    case "email": return new EmailNotifier();
    case "sms": return new SmsNotifier();
  }
}
```

### Python

```python
from abc import ABC, abstractmethod

class Notifier(ABC):
    @abstractmethod
    def send(self, message: str) -> None: ...

class EmailNotifier(Notifier):
    def send(self, message: str) -> None: ...

class SmsNotifier(Notifier):
    def send(self, message: str) -> None: ...

def create_notifier(kind: str) -> Notifier:
    match kind:
        case "email":
            return EmailNotifier()
        case "sms":
            return SmsNotifier()
        case _:
            raise ValueError(f"unknown notifier kind: {kind}")
```

### Go

```go
type Notifier interface {
    Send(message string) error
}

type emailNotifier struct{}
func (emailNotifier) Send(message string) error { return nil }

type smsNotifier struct{}
func (smsNotifier) Send(message string) error { return nil }

func NewNotifier(kind string) (Notifier, error) {
    switch kind {
    case "email":
        return emailNotifier{}, nil
    case "sms":
        return smsNotifier{}, nil
    default:
        return nil, fmt.Errorf("unknown notifier kind: %s", kind)
    }
}
```

### C#

```csharp
public interface INotifier
{
    Task SendAsync(string message);
}

public sealed class EmailNotifier : INotifier
{
    public Task SendAsync(string message) => Task.CompletedTask;
}

public sealed class SmsNotifier : INotifier
{
    public Task SendAsync(string message) => Task.CompletedTask;
}

public static class NotifierFactory
{
    public static INotifier Create(NotifierKind kind) => kind switch
    {
        NotifierKind.Email => new EmailNotifier(),
        NotifierKind.Sms => new SmsNotifier(),
        _ => throw new ArgumentOutOfRangeException(nameof(kind)),
    };
}
```

### Kotlin

```kotlin
interface Notifier {
    suspend fun send(message: String)
}

class EmailNotifier : Notifier {
    override suspend fun send(message: String) { /* ... */ }
}

class SmsNotifier : Notifier {
    override suspend fun send(message: String) { /* ... */ }
}

fun createNotifier(kind: NotifierKind): Notifier = when (kind) {
    NotifierKind.Email -> EmailNotifier()
    NotifierKind.Sms -> SmsNotifier()
}
```

### C

```c
typedef struct notifier {
    void (*send)(struct notifier *self, const char *message);
    void *state;
} notifier_t;

void email_send(notifier_t *self, const char *message) { /* ... */ }
void sms_send(notifier_t *self, const char *message) { /* ... */ }

notifier_t *notifier_create(notifier_kind_t kind) {
    notifier_t *n = malloc(sizeof(notifier_t));
    if (!n) return NULL;
    n->send = (kind == NOTIFIER_EMAIL) ? email_send : sms_send;
    n->state = NULL;
    return n;
}
/* caller owns and must call notifier_destroy(n) */
```

C has no classes, so the "factory" is a plain function returning a heap-allocated struct of function pointers (a manual vtable). Document ownership in the function name and header comment.

### C++

```cpp
class Notifier {
public:
    virtual ~Notifier() = default;
    virtual void send(std::string_view message) = 0;
};

class EmailNotifier : public Notifier {
public:
    void send(std::string_view message) override { /* ... */ }
};
class SmsNotifier : public Notifier {
public:
    void send(std::string_view message) override { /* ... */ }
};

std::unique_ptr<Notifier> make_notifier(NotifierKind kind) {
    switch (kind) {
        case NotifierKind::Email: return std::make_unique<EmailNotifier>();
        case NotifierKind::Sms:   return std::make_unique<SmsNotifier>();
    }
    throw std::invalid_argument("unknown notifier kind");
}
```

### Swift

```swift
protocol Notifier {
    func send(_ message: String) async throws
}

struct EmailNotifier: Notifier {
    func send(_ message: String) async throws { /* ... */ }
}
struct SmsNotifier: Notifier {
    func send(_ message: String) async throws { /* ... */ }
}

func makeNotifier(kind: NotifierKind) -> Notifier {
    switch kind {
    case .email: return EmailNotifier()
    case .sms: return SmsNotifier()
    }
}
```

## Pitfalls

- Adding a factory for a single concrete product "for the future" — YAGNI until a second product is real.
- Factory functions that swallow construction errors instead of returning them typed.
- Leaking the concrete product type through the return type instead of the abstraction, defeating the point of the pattern.
- Growing the factory into a god function that also does validation, logging, and business logic beyond selection.
- In GC languages, forgetting that the factory is still a good place to inject configuration/dependencies rather than reading globals inside each product.

## See Also

- [abstract-factory](abstract-factory.md) — when whole families of related products must stay compatible.
- [builder](builder.md) — when the product needs multi-step, optional-field construction rather than single-call selection.
- [dependency-injection](dependency-injection.md) — when the factory itself should be supplied from outside rather than called directly.
