# repository

> Abstract persistence behind a collection-like domain interface, so domain logic doesn't depend on a specific storage technology.

## Intent & Pressure

Reach for Repository when domain logic needs to load/save aggregates without knowing whether they live in Postgres, DynamoDB, or an in-memory test double, and when queries are expressed in domain terms ("active customers," "orders placed today") rather than raw SQL scattered through business logic. The pressure is a genuine need to swap or fake the storage layer — most commonly for tests, but also for multi-backend support or a future migration.

Do not reach for it when there is exactly one storage backend, no test-double need, and the ORM/driver's own API is already domain-appropriate — an extra abstraction layer that just forwards to the ORM one-to-one adds indirection without payoff ("repository over an ORM that's already a repository").

## Native-Construct Alternative

Call the ORM/driver directly from application/service code when there is only one backend and tests can use a real (or containerized) database. Introduce a repository interface once you need to substitute an in-memory fake in unit tests or support more than one backend.

## Language Implementations

### Rust

```rust
trait CustomerRepository {
    fn find_by_id(&self, id: CustomerId) -> Result<Option<Customer>, RepoError>;
    fn save(&self, customer: &Customer) -> Result<(), RepoError>;
}

struct PostgresCustomerRepository { pool: PgPool }
impl CustomerRepository for PostgresCustomerRepository {
    fn find_by_id(&self, id: CustomerId) -> Result<Option<Customer>, RepoError> { /* SQL */ Ok(None) }
    fn save(&self, customer: &Customer) -> Result<(), RepoError> { /* SQL */ Ok(()) }
}

// test double, no database required:
struct InMemoryCustomerRepository { customers: Mutex<HashMap<CustomerId, Customer>> }
```

### TypeScript

```typescript
interface CustomerRepository {
  findById(id: string): Promise<Customer | null>;
  save(customer: Customer): Promise<void>;
}

class PostgresCustomerRepository implements CustomerRepository {
  constructor(private pool: Pool) {}
  async findById(id: string): Promise<Customer | null> { /* SQL */ return null; }
  async save(customer: Customer): Promise<void> { /* SQL */ }
}
```

### Python

```python
from typing import Protocol

class CustomerRepository(Protocol):
    def find_by_id(self, id: str) -> Customer | None: ...
    def save(self, customer: Customer) -> None: ...

class PostgresCustomerRepository:
    def __init__(self, pool: ConnectionPool) -> None:
        self._pool = pool
    def find_by_id(self, id: str) -> Customer | None: ...  # SQL
    def save(self, customer: Customer) -> None: ...  # SQL
```

### Go

```go
type CustomerRepository interface {
    FindByID(ctx context.Context, id string) (*Customer, error)
    Save(ctx context.Context, customer *Customer) error
}

type postgresCustomerRepository struct{ pool *pgxpool.Pool }

func (r postgresCustomerRepository) FindByID(ctx context.Context, id string) (*Customer, error) {
    // SQL
    return nil, nil
}
```

### C#

```csharp
public interface ICustomerRepository
{
    Task<Customer?> FindByIdAsync(string id);
    Task SaveAsync(Customer customer);
}

public sealed class PostgresCustomerRepository : ICustomerRepository
{
    private readonly NpgsqlDataSource _dataSource;
    public PostgresCustomerRepository(NpgsqlDataSource dataSource) => _dataSource = dataSource;
    public Task<Customer?> FindByIdAsync(string id) => /* SQL */ Task.FromResult<Customer?>(null);
    public Task SaveAsync(Customer customer) => Task.CompletedTask; // SQL
}
```

Entity Framework Core's `DbSet<T>` is already repository-shaped; add a custom repository interface mainly when you need to substitute a fake in unit tests without a real `DbContext`.

### Kotlin

```kotlin
interface CustomerRepository {
    suspend fun findById(id: String): Customer?
    suspend fun save(customer: Customer)
}

class PostgresCustomerRepository(private val pool: ConnectionPool) : CustomerRepository {
    override suspend fun findById(id: String): Customer? = TODO("SQL")
    override suspend fun save(customer: Customer) = TODO("SQL")
}
```

### C

```c
typedef struct customer_repository {
    int (*find_by_id)(struct customer_repository *self, const char *id, customer_t *out);
    int (*save)(struct customer_repository *self, const customer_t *customer);
    void *state;
} customer_repository_t;

int postgres_find_by_id(customer_repository_t *self, const char *id, customer_t *out) {
    pg_conn_t *conn = self->state;
    /* SQL */
    return 0;
}
```

### C++

```cpp
class CustomerRepository {
public:
    virtual ~CustomerRepository() = default;
    virtual std::optional<Customer> findById(const std::string &id) = 0;
    virtual void save(const Customer &customer) = 0;
};

class PostgresCustomerRepository : public CustomerRepository {
public:
    explicit PostgresCustomerRepository(pqxx::connection &conn) : conn_(conn) {}
    std::optional<Customer> findById(const std::string &id) override { /* SQL */ return std::nullopt; }
    void save(const Customer &customer) override { /* SQL */ }
private:
    pqxx::connection &conn_;
};
```

### Swift

```swift
protocol CustomerRepository {
    func findById(_ id: String) async throws -> Customer?
    func save(_ customer: Customer) async throws
}

final class PostgresCustomerRepository: CustomerRepository {
    private let pool: ConnectionPool
    init(pool: ConnectionPool) { self.pool = pool }
    func findById(_ id: String) async throws -> Customer? { nil } // SQL
    func save(_ customer: Customer) async throws {} // SQL
}
```

## Pitfalls

- A repository interface that just mirrors the ORM's own query builder one-to-one, adding an indirection layer with no real substitutability benefit.
- Leaking storage-specific types (a raw SQL row, an ORM entity) through the repository interface instead of returning domain objects.
- Repositories that silently swallow persistence errors instead of returning/raising a typed failure.
- Business logic reaching past the repository to run ad hoc queries directly against the database, defeating the abstraction.
- No transactional boundary across multiple repository calls that must succeed or fail together — see [unit-of-work](unit-of-work.md).

## See Also

- [unit-of-work](unit-of-work.md) — coordinating multiple repository operations within one transaction.
- [ports-and-adapters](ports-and-adapters.md) — Repository is a specific kind of port with a persistence adapter behind it.
- [specification](specification.md) — expressing repository query criteria as composable, named predicates.
