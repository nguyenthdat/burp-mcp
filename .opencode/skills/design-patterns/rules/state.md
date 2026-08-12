# state

> Let an object's behavior change with its current state, and make every transition explicit.

## Intent & Pressure

Reach for State when an object's valid operations genuinely depend on which state it's in (an order that can only ship once paid, a TCP connection's handshake states, a media player's play/pause/stop), and the transition rules themselves are part of the domain logic worth making explicit and testable. The pressure is invalid-state prevention: without a modeled state machine, code accumulates scattered boolean flags and `if` checks that can drift out of sync and allow impossible combinations.

Do not reach for full runtime State-pattern polymorphism when the state set is closed and small — an `enum`/sealed-class plus exhaustive `match`/`when` is simpler, safer (compiler-enforced exhaustiveness), and just as expressive. Reserve a dynamic, object-per-state `State` pattern for genuinely plugin-extensible state sets.

## Native-Construct Alternative

A closed `enum`/sealed-class with an exhaustive `match`/`switch`/`when` over states and explicit transition functions returning the new state (or a typed error for invalid transitions) covers the overwhelming majority of state-machine needs, with compile-time exhaustiveness checking as a bonus.

## Language Implementations

### Rust

```rust
enum OrderState {
    Created,
    Paid { paid_at: DateTime<Utc> },
    Shipped { tracking: String },
    Cancelled { reason: String },
}

impl OrderState {
    fn pay(self, paid_at: DateTime<Utc>) -> Result<Self, TransitionError> {
        match self {
            OrderState::Created => Ok(OrderState::Paid { paid_at }),
            other => Err(TransitionError::InvalidTransition(other.name(), "pay")),
        }
    }
}
```

Consuming `self` in each transition method makes it impossible to keep using a stale state value after transitioning — a lightweight typestate discipline on top of the enum.

### TypeScript

```typescript
type OrderState =
  | { kind: "created" }
  | { kind: "paid"; paidAt: Date }
  | { kind: "shipped"; tracking: string }
  | { kind: "cancelled"; reason: string };

function pay(state: OrderState, paidAt: Date): OrderState {
  if (state.kind !== "created") {
    throw new Error(`cannot pay from state: ${state.kind}`);
  }
  return { kind: "paid", paidAt };
}
```

### Python

```python
from dataclasses import dataclass
from datetime import datetime

class OrderState: ...

@dataclass
class Created(OrderState): pass

@dataclass
class Paid(OrderState):
    paid_at: datetime

def pay(state: OrderState, paid_at: datetime) -> OrderState:
    match state:
        case Created():
            return Paid(paid_at)
        case _:
            raise ValueError(f"cannot pay from state: {state!r}")
```

### Go

```go
type OrderState interface{ isOrderState() }

type Created struct{}
func (Created) isOrderState() {}

type Paid struct{ PaidAt time.Time }
func (Paid) isOrderState() {}

func Pay(state OrderState, paidAt time.Time) (OrderState, error) {
    switch state.(type) {
    case Created:
        return Paid{PaidAt: paidAt}, nil
    default:
        return nil, fmt.Errorf("cannot pay from state: %T", state)
    }
}
```

### C#

```csharp
public abstract record OrderState;
public sealed record Created : OrderState;
public sealed record Paid(DateTimeOffset PaidAt) : OrderState;

public static OrderState Pay(OrderState state, DateTimeOffset paidAt) => state switch
{
    Created => new Paid(paidAt),
    _ => throw new InvalidOperationException($"cannot pay from state: {state.GetType().Name}"),
};
```

### Kotlin

```kotlin
sealed interface OrderState
data object Created : OrderState
data class Paid(val paidAt: Instant) : OrderState

fun pay(state: OrderState, paidAt: Instant): OrderState = when (state) {
    is Created -> Paid(paidAt)
    else -> error("cannot pay from state: $state")
}
```

### C

```c
typedef enum { ORDER_CREATED, ORDER_PAID, ORDER_SHIPPED, ORDER_CANCELLED } order_state_kind_t;

typedef struct order_state {
    order_state_kind_t kind;
    union {
        struct { time_t paid_at; } paid;
        struct { char tracking[64]; } shipped;
        struct { char reason[128]; } cancelled;
    } as;
} order_state_t;

int order_pay(order_state_t *state, time_t paid_at) {
    if (state->kind != ORDER_CREATED) return -1; /* invalid transition */
    state->kind = ORDER_PAID;
    state->as.paid.paid_at = paid_at;
    return 0;
}
```

### C++

```cpp
struct Created {};
struct Paid { std::chrono::system_clock::time_point paidAt; };
struct Shipped { std::string tracking; };
using OrderState = std::variant<Created, Paid, Shipped>;

OrderState pay(const OrderState &state, std::chrono::system_clock::time_point paidAt) {
    if (!std::holds_alternative<Created>(state)) {
        throw std::logic_error("cannot pay from current state");
    }
    return Paid{paidAt};
}
```

### Swift

```swift
enum OrderState {
    case created
    case paid(paidAt: Date)
    case shipped(tracking: String)
    case cancelled(reason: String)
}

func pay(_ state: OrderState, paidAt: Date) throws -> OrderState {
    guard case .created = state else {
        throw TransitionError.invalidTransition(from: state, to: "paid")
    }
    return .paid(paidAt: paidAt)
}
```

## Pitfalls

- Modeling state as multiple independent booleans (`isPaid`, `isShipped`, `isCancelled`) instead of one closed set, which allows impossible combinations to compile.
- Non-exhaustive transition handling that silently no-ops on an unexpected state instead of erroring.
- Storing derived/redundant data outside the state enum that can drift out of sync with the actual state.
- Runtime `State`-object polymorphism used for a small, permanently-closed state set, giving up compiler-checked exhaustiveness for no reason.
- Concurrent transitions without synchronization, allowing two threads to both "win" a transition that should be exclusive.

## See Also

- [strategy](strategy.md) — varying an algorithm regardless of state, versus varying behavior specifically by current state.
- [observer](observer.md) — notifying dependents when a state transition happens.
- [memento](memento.md) — capturing a state snapshot for later restoration.
