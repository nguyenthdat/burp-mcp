# dependency-injection

> Supply a component's collaborators from outside rather than having it construct them internally, so callers can vary and test them independently.

## Intent & Pressure

Reach for Dependency Injection whenever a component's collaborators vary by environment (production vs. test), configuration, or deployment, and internal construction would hard-code a specific implementation the caller can't substitute. The pressure is testability and flexibility: code that `new`s its own database client, HTTP client, or clock internally cannot be tested without hitting the real thing, and cannot be reconfigured without editing its source.

Do not reach for a DI *framework*/container for a small object graph — manual constructor injection wired up once in `main`/the composition root is simpler, faster to understand, and has no runtime magic. Reserve a framework for large graphs with many optional/conditional dependencies where manual wiring becomes its own maintenance burden.

## Native-Construct Alternative

Pass the dependency as a constructor or function parameter ("poor man's DI"), wired once at the composition root (`main`, app startup, test setup). This handles the overwhelming majority of real-world DI needs without any container/framework at all.

## Language Implementations

### Rust

```rust
struct OrderService<C: Clock, R: Repository> {
    clock: C,
    repo: R,
}

impl<C: Clock, R: Repository> OrderService<C, R> {
    fn new(clock: C, repo: R) -> Self { Self { clock, repo } }
    fn place_order(&self, order: NewOrder) -> Result<OrderId, OrderError> {
        let now = self.clock.now();
        self.repo.save(order.with_timestamp(now))
    }
}

// wiring, once, at the composition root:
let service = OrderService::new(SystemClock, PostgresRepository::connect(&url)?);
```

Generic constructor parameters give compile-time-checked, zero-cost DI; use `Box<dyn Trait>` fields only when the concrete implementation must be chosen at runtime.

### TypeScript

```typescript
interface Clock { now(): Date; }
interface Repository { save(order: Order): Promise<string>; }

class OrderService {
  constructor(private clock: Clock, private repo: Repository) {}
  async placeOrder(order: NewOrder): Promise<string> {
    return this.repo.save({ ...order, timestamp: this.clock.now() });
  }
}

// wiring, once, at the composition root:
const service = new OrderService(new SystemClock(), new PostgresRepository(url));
```

### Python

```python
class OrderService:
    def __init__(self, clock: Clock, repo: Repository) -> None:
        self._clock = clock
        self._repo = repo

    def place_order(self, order: NewOrder) -> str:
        return self._repo.save(order.with_timestamp(self._clock.now()))

# wiring, once, at the composition root:
service = OrderService(SystemClock(), PostgresRepository(url))
```

### Go

```go
type OrderService struct {
    clock Clock
    repo  Repository
}

func NewOrderService(clock Clock, repo Repository) *OrderService {
    return &OrderService{clock: clock, repo: repo}
}

func (s *OrderService) PlaceOrder(order NewOrder) (string, error) {
    return s.repo.Save(order.WithTimestamp(s.clock.Now()))
}

// wiring, once, in main():
service := NewOrderService(SystemClock{}, NewPostgresRepository(url))
```

### C#

```csharp
public sealed class OrderService
{
    private readonly IClock _clock;
    private readonly IRepository _repo;
    public OrderService(IClock clock, IRepository repo) { _clock = clock; _repo = repo; }

    public Task<string> PlaceOrderAsync(NewOrder order) =>
        _repo.SaveAsync(order.WithTimestamp(_clock.Now()));
}

// wiring via the built-in DI container:
services.AddSingleton<IClock, SystemClock>();
services.AddScoped<IRepository, PostgresRepository>();
services.AddScoped<OrderService>();
```

### Kotlin

```kotlin
class OrderService(private val clock: Clock, private val repo: Repository) {
    suspend fun placeOrder(order: NewOrder): String =
        repo.save(order.withTimestamp(clock.now()))
}

// manual wiring, or via Hilt/Koin annotations in larger apps:
val service = OrderService(SystemClock(), PostgresRepository(url))
```

### C

```c
typedef struct order_service {
    clock_t     *clock;
    repository_t *repo;
} order_service_t;

order_service_t order_service_create(clock_t *clock, repository_t *repo) {
    return (order_service_t){ .clock = clock, .repo = repo };
}

int order_service_place_order(order_service_t *svc, const new_order_t *order, char *id_out) {
    time_t now = svc->clock->now(svc->clock);
    return repository_save(svc->repo, order, now, id_out);
}
```

C's "DI" is passing struct/function pointers explicitly at every call site — there is no container, just discipline about not reaching for globals.

### C++

```cpp
class OrderService {
public:
    OrderService(std::shared_ptr<Clock> clock, std::shared_ptr<Repository> repo)
        : clock_(std::move(clock)), repo_(std::move(repo)) {}

    std::string placeOrder(const NewOrder &order) {
        return repo_->save(order.withTimestamp(clock_->now()));
    }
private:
    std::shared_ptr<Clock> clock_;
    std::shared_ptr<Repository> repo_;
};

// wiring, once, in main():
auto service = OrderService(std::make_shared<SystemClock>(), std::make_shared<PostgresRepository>(url));
```

### Swift

```swift
final class OrderService {
    private let clock: Clock
    private let repo: Repository
    init(clock: Clock, repo: Repository) { self.clock = clock; self.repo = repo }

    func placeOrder(_ order: NewOrder) async throws -> String {
        try await repo.save(order.withTimestamp(clock.now()))
    }
}

// wiring, once, at app startup:
let service = OrderService(clock: SystemClock(), repo: PostgresRepository(url: url))
```

## Pitfalls

- Reaching for a DI framework/container before the object graph is complex enough to need one — manual constructor wiring is usually clearer.
- Service locator anti-pattern (a global registry components pull dependencies *from*) instead of true injection — it hides the dependency just as much as a Singleton does.
- Constructor parameter lists growing unbounded instead of grouping related dependencies into a cohesive collaborator.
- Injecting concrete types instead of the abstraction the consumer actually needs, defeating substitutability in tests.
- Circular dependencies between injected components, which most containers will only catch at runtime, not compile time.

## See Also

- [singleton](singleton.md) — the anti-pattern DI exists to replace for shared, hard-coded dependencies.
- [factory-method](factory-method.md) — often what a DI container calls internally to construct registered types.
- [ports-and-adapters](ports-and-adapters.md) — DI is the mechanism that wires adapters into domain-facing ports.
