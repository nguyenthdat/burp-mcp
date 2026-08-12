# event-sourcing

> Persist state as an append-only sequence of domain events, reconstructing current state by replaying them, instead of storing only the latest snapshot.

## Intent & Pressure

Reach for Event Sourcing when the system genuinely needs a full audit trail of every state change, the ability to answer "what did this look like at time T," or event-driven integration where downstream consumers care about *what happened*, not just the current value. The pressure is a real requirement for history/replay/audit as a first-class feature, not a "nice to have."

Do not reach for it for typical CRUD needs where a current-state table plus an audit log/history table already satisfies compliance and debugging needs — Event Sourcing adds substantial complexity (event versioning/schema evolution, replay performance, snapshotting, eventual consistency with any read projections) that is expensive to carry without a genuine need for it.

## Native-Construct Alternative

Standard CRUD with an append-only audit/history table alongside the current-state table covers "we need to know what changed and when" for the majority of systems, without needing to reconstruct state by replaying events on every read.

## Language Implementations

### Rust

```rust
enum AccountEvent {
    Opened { owner: CustomerId },
    Deposited { amount: Money },
    Withdrawn { amount: Money },
}

#[derive(Default)]
struct Account { balance: Money, owner: Option<CustomerId> }

impl Account {
    fn apply(&mut self, event: &AccountEvent) {
        match event {
            AccountEvent::Opened { owner } => self.owner = Some(*owner),
            AccountEvent::Deposited { amount } => self.balance += *amount,
            AccountEvent::Withdrawn { amount } => self.balance -= *amount,
        }
    }
    fn replay(events: &[AccountEvent]) -> Self {
        let mut account = Self::default();
        for event in events { account.apply(event); }
        account
    }
}
```

### TypeScript

```typescript
type AccountEvent =
  | { kind: "opened"; owner: string }
  | { kind: "deposited"; amount: number }
  | { kind: "withdrawn"; amount: number };

function apply(account: Account, event: AccountEvent): Account {
  switch (event.kind) {
    case "opened": return { ...account, owner: event.owner };
    case "deposited": return { ...account, balance: account.balance + event.amount };
    case "withdrawn": return { ...account, balance: account.balance - event.amount };
  }
}

function replay(events: AccountEvent[]): Account {
  return events.reduce(apply, { balance: 0, owner: null });
}
```

### Python

```python
from dataclasses import dataclass, replace

class AccountEvent: ...

@dataclass
class Deposited(AccountEvent):
    amount: Decimal

def apply(account: Account, event: AccountEvent) -> Account:
    match event:
        case Deposited(amount):
            return replace(account, balance=account.balance + amount)
        case _:
            raise TypeError(f"unhandled event: {event!r}")

def replay(events: list[AccountEvent]) -> Account:
    account = Account(balance=Decimal(0), owner=None)
    for event in events:
        account = apply(account, event)
    return account
```

### Go

```go
type AccountEvent interface{ isAccountEvent() }

type Deposited struct{ Amount int64 }
func (Deposited) isAccountEvent() {}

func Apply(account Account, event AccountEvent) Account {
    switch e := event.(type) {
    case Deposited:
        account.Balance += e.Amount
    }
    return account
}

func Replay(events []AccountEvent) Account {
    var account Account
    for _, e := range events {
        account = Apply(account, e)
    }
    return account
}
```

### C#

```csharp
public abstract record AccountEvent;
public sealed record Deposited(decimal Amount) : AccountEvent;
public sealed record Withdrawn(decimal Amount) : AccountEvent;

public static Account Apply(Account account, AccountEvent evt) => evt switch
{
    Deposited d => account with { Balance = account.Balance + d.Amount },
    Withdrawn w => account with { Balance = account.Balance - w.Amount },
    _ => account,
};

public static Account Replay(IEnumerable<AccountEvent> events) =>
    events.Aggregate(new Account(), Apply);
```

### Kotlin

```kotlin
sealed interface AccountEvent
data class Deposited(val amount: BigDecimal) : AccountEvent
data class Withdrawn(val amount: BigDecimal) : AccountEvent

fun apply(account: Account, event: AccountEvent): Account = when (event) {
    is Deposited -> account.copy(balance = account.balance + event.amount)
    is Withdrawn -> account.copy(balance = account.balance - event.amount)
}

fun replay(events: List<AccountEvent>): Account =
    events.fold(Account()) { account, event -> apply(account, event) }
```

### C

```c
typedef enum { EVENT_DEPOSITED, EVENT_WITHDRAWN } account_event_kind_t;
typedef struct { account_event_kind_t kind; int64_t amount; } account_event_t;

void account_apply(account_t *account, const account_event_t *event) {
    switch (event->kind) {
        case EVENT_DEPOSITED: account->balance += event->amount; break;
        case EVENT_WITHDRAWN: account->balance -= event->amount; break;
    }
}

account_t account_replay(const account_event_t *events, size_t count) {
    account_t account = {0};
    for (size_t i = 0; i < count; i++) account_apply(&account, &events[i]);
    return account;
}
```

### C++

```cpp
struct Deposited { std::int64_t amount; };
struct Withdrawn { std::int64_t amount; };
using AccountEvent = std::variant<Deposited, Withdrawn>;

Account apply(Account account, const AccountEvent &event) {
    std::visit(overloaded{
        [&](const Deposited &d) { account.balance += d.amount; },
        [&](const Withdrawn &w) { account.balance -= w.amount; },
    }, event);
    return account;
}

Account replay(const std::vector<AccountEvent> &events) {
    return std::accumulate(events.begin(), events.end(), Account{}, apply);
}
```

### Swift

```swift
enum AccountEvent {
    case deposited(amount: Decimal)
    case withdrawn(amount: Decimal)
}

func apply(_ account: Account, _ event: AccountEvent) -> Account {
    switch event {
    case .deposited(let amount):
        var updated = account; updated.balance += amount; return updated
    case .withdrawn(let amount):
        var updated = account; updated.balance -= amount; return updated
    }
}

func replay(_ events: [AccountEvent]) -> Account {
    events.reduce(Account(), apply)
}
```

## Pitfalls

- No event schema versioning strategy, so a code change breaks replay of historical events (add upcasting/migration for old event shapes, never silently reinterpret them).
- Replaying the full event stream on every read instead of periodic snapshotting, causing replay time to grow unbounded with history.
- Non-deterministic event application (using current time, random values, or external I/O inside `apply`) breaking reproducible replay.
- Treating events as internal implementation detail and changing their shape freely, when other services may already depend on published event schemas.
- No dead-letter/retry strategy for downstream projections that fail to apply an event, silently leaving read models stale or inconsistent.

## See Also

- [cqrs](cqrs.md) — the natural read-side companion: events populate denormalized query projections.
- [memento](memento.md) — snapshotting state directly, versus reconstructing it from a full event history.
- [pub-sub](pub-sub.md) — the transport mechanism events are typically published through to other services.
