# ports-and-adapters

> Isolate domain logic behind narrow ports it defines, with infrastructure implemented as replaceable adapters outside the domain (hexagonal architecture).

## Intent & Pressure

Reach for Ports and Adapters when domain logic must stay testable and framework-independent — able to run against an in-memory fake in unit tests and against real infrastructure (database, message queue, HTTP API) in production, without the domain layer importing infrastructure libraries directly. The pressure is architectural: without this boundary, domain logic accumulates direct dependencies on ORMs, HTTP clients, and SDKs, making it slow to test and hard to change infrastructure later.

Do not reach for a full hexagonal architecture on a small script or a CRUD service with one obvious infrastructure integration and no swap requirement — the port/adapter/DI ceremony costs real setup for a payoff that only shows up at a certain size and lifetime. Introduce it when the domain logic is complex enough, or long-lived enough, that framework independence and test speed matter.

## Native-Construct Alternative

A thin service layer that directly calls the ORM/HTTP client is fine for small services with a single integration. Introduce explicit ports (interfaces owned by the domain) and adapters (infrastructure implementations of those interfaces) once you need to substitute fakes in tests or swap the underlying infrastructure.

## Language Implementations

### Rust

```rust
// Port: defined by the domain, in domain terms
trait PaymentGateway {
    fn charge(&self, amount: Money, card: &CardToken) -> Result<ChargeId, PaymentError>;
}

// Domain logic depends only on the port
struct CheckoutService<G: PaymentGateway> { gateway: G }
impl<G: PaymentGateway> CheckoutService<G> {
    fn checkout(&self, cart: Cart, card: CardToken) -> Result<ChargeId, PaymentError> {
        self.gateway.charge(cart.total(), &card)
    }
}

// Adapter: infrastructure implementation, outside the domain module
struct StripeAdapter { client: StripeClient }
impl PaymentGateway for StripeAdapter {
    fn charge(&self, amount: Money, card: &CardToken) -> Result<ChargeId, PaymentError> { /* Stripe SDK call */ Ok(ChargeId(0)) }
}
```

### TypeScript

```typescript
// Port
interface PaymentGateway {
  charge(amount: Money, card: CardToken): Promise<string>;
}

// Domain logic
class CheckoutService {
  constructor(private gateway: PaymentGateway) {}
  async checkout(cart: Cart, card: CardToken): Promise<string> {
    return this.gateway.charge(cart.total(), card);
  }
}

// Adapter
class StripeAdapter implements PaymentGateway {
  constructor(private client: Stripe) {}
  async charge(amount: Money, card: CardToken): Promise<string> { /* Stripe SDK call */ return ""; }
}
```

### Python

```python
class PaymentGateway(Protocol):
    def charge(self, amount: Money, card: CardToken) -> str: ...

class CheckoutService:
    def __init__(self, gateway: PaymentGateway) -> None:
        self._gateway = gateway

    def checkout(self, cart: Cart, card: CardToken) -> str:
        return self._gateway.charge(cart.total(), card)

class StripeAdapter:
    def __init__(self, client: stripe.Client) -> None:
        self._client = client
    def charge(self, amount: Money, card: CardToken) -> str: ...  # Stripe SDK call
```

### Go

```go
// Port, defined alongside the domain
type PaymentGateway interface {
    Charge(ctx context.Context, amount Money, card CardToken) (string, error)
}

type CheckoutService struct{ gateway PaymentGateway }

func (s CheckoutService) Checkout(ctx context.Context, cart Cart, card CardToken) (string, error) {
    return s.gateway.Charge(ctx, cart.Total(), card)
}

// Adapter, in an infra package importing the Stripe SDK
type StripeAdapter struct{ client *stripe.Client }

func (a StripeAdapter) Charge(ctx context.Context, amount Money, card CardToken) (string, error) {
    return "", nil // Stripe SDK call
}
```

### C#

```csharp
public interface IPaymentGateway
{
    Task<string> ChargeAsync(Money amount, CardToken card);
}

public sealed class CheckoutService
{
    private readonly IPaymentGateway _gateway;
    public CheckoutService(IPaymentGateway gateway) => _gateway = gateway;

    public Task<string> CheckoutAsync(Cart cart, CardToken card) =>
        _gateway.ChargeAsync(cart.Total(), card);
}

public sealed class StripeAdapter : IPaymentGateway
{
    private readonly StripeClient _client;
    public StripeAdapter(StripeClient client) => _client = client;
    public Task<string> ChargeAsync(Money amount, CardToken card) => Task.FromResult(""); // Stripe SDK
}
```

### Kotlin

```kotlin
interface PaymentGateway {
    suspend fun charge(amount: Money, card: CardToken): String
}

class CheckoutService(private val gateway: PaymentGateway) {
    suspend fun checkout(cart: Cart, card: CardToken): String = gateway.charge(cart.total(), card)
}

class StripeAdapter(private val client: StripeClient) : PaymentGateway {
    override suspend fun charge(amount: Money, card: CardToken): String = "" // Stripe SDK call
}
```

### C

```c
typedef struct payment_gateway {
    int (*charge)(struct payment_gateway *self, int64_t amount_cents, const char *card_token, char *charge_id_out);
    void *state;
} payment_gateway_t;

int checkout(payment_gateway_t *gateway, const cart_t *cart, const char *card_token, char *charge_id_out) {
    return gateway->charge(gateway, cart_total(cart), card_token, charge_id_out);
}

/* Adapter, in a separate infra_stripe.c translation unit */
int stripe_charge(payment_gateway_t *self, int64_t amount_cents, const char *card_token, char *out) {
    /* Stripe SDK call */
    return 0;
}
```

### C++

```cpp
class PaymentGateway {
public:
    virtual ~PaymentGateway() = default;
    virtual std::string charge(Money amount, const CardToken &card) = 0;
};

class CheckoutService {
public:
    explicit CheckoutService(std::shared_ptr<PaymentGateway> gateway) : gateway_(std::move(gateway)) {}
    std::string checkout(const Cart &cart, const CardToken &card) {
        return gateway_->charge(cart.total(), card);
    }
private:
    std::shared_ptr<PaymentGateway> gateway_;
};

class StripeAdapter : public PaymentGateway {
public:
    explicit StripeAdapter(StripeClient &client) : client_(client) {}
    std::string charge(Money amount, const CardToken &card) override { return ""; } // Stripe SDK
private:
    StripeClient &client_;
};
```

### Swift

```swift
protocol PaymentGateway {
    func charge(amount: Money, card: CardToken) async throws -> String
}

struct CheckoutService {
    let gateway: PaymentGateway
    func checkout(cart: Cart, card: CardToken) async throws -> String {
        try await gateway.charge(amount: cart.total(), card: card)
    }
}

struct StripeAdapter: PaymentGateway {
    let client: StripeClient
    func charge(amount: Money, card: CardToken) async throws -> String { "" } // Stripe SDK call
}
```

## Pitfalls

- Letting domain code import an infrastructure SDK/ORM type directly, even "just this once" — that's the exact coupling this pattern exists to prevent.
- Defining ports too broadly (mirroring an entire third-party SDK) instead of narrow, domain-shaped interfaces the domain actually needs.
- Introducing this architecture on a small, short-lived service where the setup cost outweighs the testability/framework-independence benefit.
- Adapters that leak infrastructure-specific error types through the port instead of translating them to domain errors.
- Treating "port" as a synonym for "any interface" — a port is specifically domain-owned and infrastructure-agnostic.

## See Also

- [dependency-injection](dependency-injection.md) — the mechanism that wires concrete adapters into domain-facing ports.
- [adapter](adapter.md) — the structural pattern this architecture applies at the infrastructure boundary.
- [repository](repository.md) — a common specific port for persistence.
