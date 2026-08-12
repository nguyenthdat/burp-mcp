# cqrs

> Separate the model and code path used to change state (commands) from the model and code path used to read it (queries).

## Intent & Pressure

Reach for CQRS when read and write workloads have genuinely different shapes, scaling needs, or consistency requirements — reads need denormalized, join-heavy, highly-cached views serving high volume, while writes need strict validation and a normalized model enforcing invariants. The pressure is a real mismatch: one model straining to serve both a demanding query pattern and strict write invariants at once.

Do not reach for it for a typical CRUD service where the same model serves reads and writes just fine — CQRS adds real cost (eventual consistency between write and read models, more moving parts, synchronization logic) that is not justified without a demonstrated read/write mismatch. Full CQRS with separate data stores is a bigger step than "separate command and query methods" — start with the latter.

## Native-Construct Alternative

Separate command and query *methods* (or even just separate function names) operating on the same model and same store is "CQRS" in its lightest form and is sufficient for most services — it improves clarity without introducing a second data store or synchronization pipeline. Escalate to a fully separate read model/store only when the read workload demonstrably can't be served well by the write model.

## Language Implementations

### Rust

```rust
// Command side: enforces invariants, normalized model
struct CreateOrderCommand { customer_id: CustomerId, items: Vec<OrderItem> }

async fn handle_create_order(db: &PgPool, cmd: CreateOrderCommand) -> Result<OrderId, OrderError> {
    let mut tx = db.begin().await?;
    let order_id = insert_order(&mut tx, &cmd).await?;
    tx.commit().await?;
    Ok(order_id)
}

// Query side: denormalized, read-optimized, possibly a different store
async fn get_order_summary(read_db: &PgPool, id: OrderId) -> Result<OrderSummaryView, QueryError> {
    sqlx::query_as!(OrderSummaryView, "SELECT * FROM order_summary_view WHERE id = $1", id.0)
        .fetch_one(read_db).await.map_err(Into::into)
}
```

### TypeScript

```typescript
// Command
async function createOrder(cmd: CreateOrderCommand): Promise<string> {
  const orderId = await writeDb.insertOrder(cmd);
  await eventBus.publish(new OrderCreated(orderId, cmd));
  return orderId;
}

// Query — reads from a denormalized projection, not the write model
async function getOrderSummary(id: string): Promise<OrderSummaryView> {
  return readDb.orderSummaryViews.findById(id);
}
```

### Python

```python
# Command
def create_order(cmd: CreateOrderCommand) -> str:
    order_id = write_repo.insert_order(cmd)
    event_bus.publish(OrderCreated(order_id, cmd))
    return order_id

# Query
def get_order_summary(order_id: str) -> OrderSummaryView:
    return read_repo.find_summary(order_id)
```

### Go

```go
// Command
func CreateOrder(ctx context.Context, cmd CreateOrderCommand) (string, error) {
    orderID, err := writeRepo.InsertOrder(ctx, cmd)
    if err != nil {
        return "", err
    }
    eventBus.Publish(ctx, OrderCreated{OrderID: orderID})
    return orderID, nil
}

// Query
func GetOrderSummary(ctx context.Context, id string) (OrderSummaryView, error) {
    return readRepo.FindSummary(ctx, id)
}
```

### C#

```csharp
// Command handler (e.g., via MediatR)
public sealed class CreateOrderHandler
{
    public async Task<string> Handle(CreateOrderCommand cmd)
    {
        var orderId = await _writeRepo.InsertOrderAsync(cmd);
        await _eventBus.PublishAsync(new OrderCreated(orderId));
        return orderId;
    }
}

// Query handler — reads a separate projection/view
public sealed class GetOrderSummaryHandler
{
    public Task<OrderSummaryView> Handle(GetOrderSummaryQuery query) =>
        _readRepo.FindSummaryAsync(query.OrderId);
}
```

### Kotlin

```kotlin
// Command
suspend fun createOrder(cmd: CreateOrderCommand): String {
    val orderId = writeRepo.insertOrder(cmd)
    eventBus.publish(OrderCreated(orderId))
    return orderId
}

// Query
suspend fun getOrderSummary(id: String): OrderSummaryView = readRepo.findSummary(id)
```

### C

```c
/* Command */
int create_order(const create_order_cmd_t *cmd, char *order_id_out) {
    if (write_repo_insert_order(cmd, order_id_out) != 0) return -1;
    event_bus_publish(EVENT_ORDER_CREATED, order_id_out);
    return 0;
}

/* Query */
int get_order_summary(const char *order_id, order_summary_view_t *out) {
    return read_repo_find_summary(order_id, out);
}
```

### C++

```cpp
// Command
std::string createOrder(const CreateOrderCommand &cmd) {
    auto orderId = writeRepo.insertOrder(cmd);
    eventBus.publish(OrderCreated{orderId});
    return orderId;
}

// Query
OrderSummaryView getOrderSummary(const std::string &orderId) {
    return readRepo.findSummary(orderId);
}
```

### Swift

```swift
// Command
func createOrder(_ cmd: CreateOrderCommand) async throws -> String {
    let orderId = try await writeRepo.insertOrder(cmd)
    try await eventBus.publish(OrderCreated(orderId: orderId))
    return orderId
}

// Query
func getOrderSummary(_ id: String) async throws -> OrderSummaryView {
    try await readRepo.findSummary(id)
}
```

## Pitfalls

- Adopting a fully separate read store/pipeline for a service with no demonstrated read/write mismatch — the eventual-consistency and synchronization overhead isn't free.
- Callers reading the write model directly for convenience, bypassing the read model and reintroducing the coupling CQRS was meant to remove.
- No documented staleness bound on the read model, surprising callers who expect read-after-write consistency.
- Command handlers that also try to return rich query-shaped data, blurring the separation and coupling the two sides' schemas together.
- Treating "commands" as just a naming convention with no actual validation/invariant enforcement, losing the main benefit of separating them from queries.

## See Also

- [event-sourcing](event-sourcing.md) — a common (but not required) companion: events from the write side populate read-model projections.
- [repository](repository.md) — often used on both sides, with different query shapes.
- [unit-of-work](unit-of-work.md) — coordinating the write side's transactional consistency.
