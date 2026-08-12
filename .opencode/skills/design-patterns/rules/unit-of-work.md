# unit-of-work

> Track a batch of related changes across one or more repositories and commit or roll them all back together as one transaction.

## Intent & Pressure

Reach for Unit of Work when a single business operation touches multiple aggregates/repositories that must succeed or fail atomically (debit one account and credit another; update an order and decrement inventory), and each repository doesn't know about the others' state. The pressure is transactional consistency across operations that individually look independent but aren't.

Do not reach for it when a single repository call is already transactional on its own (most single-aggregate saves are) — adding a Unit of Work around one call is ceremony with no benefit. Reserve it for genuinely multi-repository, multi-step operations that need one commit/rollback boundary.

## Native-Construct Alternative

The database/ORM's own transaction/session scope (a `with` block, a `using` statement, a context manager) is frequently sufficient without a custom Unit of Work abstraction layered on top — reach for a dedicated type mainly when you need to swap the transactional mechanism (real DB vs. in-memory fake) behind one interface for testing.

## Language Implementations

### Rust

```rust
struct UnitOfWork<'a> {
    tx: sqlx::Transaction<'a, sqlx::Postgres>,
}

impl<'a> UnitOfWork<'a> {
    async fn transfer(&mut self, from: AccountId, to: AccountId, amount: Money) -> Result<(), TransferError> {
        debit_account(&mut *self.tx, from, amount).await?;
        credit_account(&mut *self.tx, to, amount).await?;
        Ok(())
    }
    async fn commit(self) -> Result<(), sqlx::Error> { self.tx.commit().await }
}

// usage: a UnitOfWork owns the transaction; dropping it without commit rolls back.
```

### TypeScript

```typescript
async function transfer(pool: Pool, from: string, to: string, amount: number): Promise<void> {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");
    await debitAccount(client, from, amount);
    await creditAccount(client, to, amount);
    await client.query("COMMIT");
  } catch (err) {
    await client.query("ROLLBACK");
    throw err;
  } finally {
    client.release();
  }
}
```

### Python

```python
def transfer(session: Session, from_id: str, to_id: str, amount: Decimal) -> None:
    with session.begin():  # commits on success, rolls back on exception
        debit_account(session, from_id, amount)
        credit_account(session, to_id, amount)
```

SQLAlchemy's `Session` already implements Unit of Work: it tracks pending changes and flushes/commits them together.

### Go

```go
func Transfer(ctx context.Context, db *sql.DB, from, to string, amount int64) error {
    tx, err := db.BeginTx(ctx, nil)
    if err != nil {
        return err
    }
    defer tx.Rollback() // no-op if already committed

    if err := debitAccount(ctx, tx, from, amount); err != nil {
        return err
    }
    if err := creditAccount(ctx, tx, to, amount); err != nil {
        return err
    }
    return tx.Commit()
}
```

### C#

```csharp
public async Task TransferAsync(string fromId, string toId, decimal amount)
{
    await using var transaction = await _dbContext.Database.BeginTransactionAsync();
    try
    {
        await _accounts.DebitAsync(fromId, amount);
        await _accounts.CreditAsync(toId, amount);
        await _dbContext.SaveChangesAsync();
        await transaction.CommitAsync();
    }
    catch
    {
        await transaction.RollbackAsync();
        throw;
    }
}
```

EF Core's `DbContext.ChangeTracker` plus `SaveChangesAsync` is itself a Unit of Work; an explicit transaction is only needed when multiple `SaveChanges` calls or multiple contexts must be atomic together.

### Kotlin

```kotlin
suspend fun transfer(db: Database, fromId: String, toId: String, amount: BigDecimal) {
    newSuspendedTransaction(db = db) { // Exposed: commits on success, rolls back on exception
        debitAccount(fromId, amount)
        creditAccount(toId, amount)
    }
}
```

### C

```c
int transfer(pg_conn_t *conn, const char *from_id, const char *to_id, int64_t amount) {
    if (pg_exec(conn, "BEGIN") != 0) return -1;

    if (debit_account(conn, from_id, amount) != 0) { pg_exec(conn, "ROLLBACK"); return -1; }
    if (credit_account(conn, to_id, amount) != 0) { pg_exec(conn, "ROLLBACK"); return -1; }

    return pg_exec(conn, "COMMIT");
}
```

### C++

```cpp
void transfer(pqxx::connection &conn, const std::string &fromId, const std::string &toId, long amountCents) {
    pqxx::work tx(conn); // RAII: rolls back on destruction unless commit() is called
    debitAccount(tx, fromId, amountCents);
    creditAccount(tx, toId, amountCents);
    tx.commit();
}
```

`pqxx::work` (and similar RAII transaction wrappers) automatically roll back if an exception propagates before `commit()` — the C++ idiom for this pattern.

### Swift

```swift
func transfer(db: Database, fromId: String, toId: String, amount: Decimal) async throws {
    try await db.transaction { transaction in
        try await debitAccount(transaction, fromId, amount)
        try await creditAccount(transaction, toId, amount)
    } // commits if the closure returns normally, rolls back if it throws
}
```

## Pitfalls

- Wrapping a single repository call in a Unit of Work for no reason, adding ceremony without a real cross-aggregate transaction need.
- Holding a transaction open across a slow external call (network request, user interaction), causing long-held database locks.
- Forgetting to roll back on early return/exception paths — always use RAII/`using`/`defer`/context-manager mechanics rather than manual commit/rollback bookkeeping.
- Mixing units of work across different databases/data stores that don't share a real distributed-transaction mechanism, silently losing atomicity.
- Retrying a unit of work that already partially committed side effects outside the transaction (e.g., sent an email) — separate transactional and non-transactional side effects clearly.

## See Also

- [repository](repository.md) — the collaborators a Unit of Work typically coordinates.
- [event-sourcing](event-sourcing.md) — an alternative consistency model based on append-only events rather than transactional row updates.
- [cqrs](cqrs.md) — often paired: the write side uses Unit of Work, the read side does not need to.
