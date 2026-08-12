# proxy

> Control access to a real service behind the same contract it exposes, adding authorization, caching, lazy loading, rate limiting, retries, or remote indirection.

## Intent & Pressure

Reach for Proxy when callers must go through the real object's exact interface, but something must happen around every call — an auth check, a cache lookup, lazy construction of an expensive resource, or marshaling to a remote process. The pressure is: the contract must stay identical to the real subject, so all existing callers keep working unmodified.

Do not reach for it when you can add the cross-cutting behavior directly to the real implementation, or when only one call site needs it (inline the check there instead). Do not confuse Proxy with Decorator: Proxy controls/gates access to the *same* operation, Decorator adds *new* behavior/data on top of it.

## Native-Construct Alternative

An inline check at the single call site (`if !authorized { return Err(...) }`) is simpler when there's exactly one caller. Promote to a Proxy type implementing the shared interface once multiple callers need the same policy applied transparently.

## Language Implementations

### Rust

```rust
trait Repository {
    fn get(&self, id: Id) -> Result<Record, RepoError>;
}

struct CachingRepository<R: Repository> {
    inner: R,
    cache: Mutex<HashMap<Id, Record>>,
}

impl<R: Repository> Repository for CachingRepository<R> {
    fn get(&self, id: Id) -> Result<Record, RepoError> {
        if let Some(record) = self.cache.lock().unwrap().get(&id) {
            return Ok(record.clone());
        }
        let record = self.inner.get(id)?;
        self.cache.lock().unwrap().insert(id, record.clone());
        Ok(record)
    }
}
```

### TypeScript

```typescript
interface Repository {
  get(id: string): Promise<Record>;
}

class CachingRepository implements Repository {
  private cache = new Map<string, Record>();
  constructor(private inner: Repository) {}

  async get(id: string): Promise<Record> {
    const cached = this.cache.get(id);
    if (cached) return cached;
    const record = await this.inner.get(id);
    this.cache.set(id, record);
    return record;
  }
}
```

### Python

```python
class CachingRepository:
    def __init__(self, inner: Repository) -> None:
        self._inner = inner
        self._cache: dict[str, Record] = {}

    def get(self, id: str) -> Record:
        if id in self._cache:
            return self._cache[id]
        record = self._inner.get(id)
        self._cache[id] = record
        return record
```

### Go

```go
type cachingRepository struct {
    inner Repository
    mu    sync.Mutex
    cache map[string]Record
}

func (c *cachingRepository) Get(id string) (Record, error) {
    c.mu.Lock()
    if record, ok := c.cache[id]; ok {
        c.mu.Unlock()
        return record, nil
    }
    c.mu.Unlock()

    record, err := c.inner.Get(id)
    if err != nil {
        return Record{}, err
    }
    c.mu.Lock()
    c.cache[id] = record
    c.mu.Unlock()
    return record, nil
}
```

### C#

```csharp
public sealed class CachingRepository : IRepository
{
    private readonly IRepository _inner;
    private readonly ConcurrentDictionary<string, Record> _cache = new();
    public CachingRepository(IRepository inner) => _inner = inner;

    public async Task<Record> GetAsync(string id)
    {
        if (_cache.TryGetValue(id, out var cached)) return cached;
        var record = await _inner.GetAsync(id);
        _cache[id] = record;
        return record;
    }
}
```

### Kotlin

```kotlin
class CachingRepository(private val inner: Repository) : Repository {
    private val cache = ConcurrentHashMap<String, Record>()

    override suspend fun get(id: String): Record =
        cache[id] ?: inner.get(id).also { cache[id] = it }
}
```

### C

```c
typedef struct caching_repository {
    repository_t *inner;
    hash_map_t   *cache; /* id -> record_t* */
    pthread_mutex_t lock;
} caching_repository_t;

int caching_repository_get(caching_repository_t *self, const char *id, record_t *out) {
    pthread_mutex_lock(&self->lock);
    record_t *cached = hash_map_get(self->cache, id);
    pthread_mutex_unlock(&self->lock);
    if (cached) { *out = *cached; return 0; }

    if (repository_get(self->inner, id, out) != 0) return -1;

    pthread_mutex_lock(&self->lock);
    hash_map_put(self->cache, id, out);
    pthread_mutex_unlock(&self->lock);
    return 0;
}
```

### C++

```cpp
class Repository {
public:
    virtual ~Repository() = default;
    virtual Record get(const std::string &id) = 0;
};

class CachingRepository : public Repository {
public:
    explicit CachingRepository(std::unique_ptr<Repository> inner) : inner_(std::move(inner)) {}

    Record get(const std::string &id) override {
        std::lock_guard lock(mutex_);
        auto it = cache_.find(id);
        if (it != cache_.end()) return it->second;
        auto record = inner_->get(id);
        cache_.emplace(id, record);
        return record;
    }
private:
    std::unique_ptr<Repository> inner_;
    std::unordered_map<std::string, Record> cache_;
    std::mutex mutex_;
};
```

### Swift

```swift
actor CachingRepository: Repository {
    private let inner: Repository
    private var cache: [String: Record] = [:]

    init(inner: Repository) { self.inner = inner }

    func get(_ id: String) async throws -> Record {
        if let cached = cache[id] { return cached }
        let record = try await inner.get(id)
        cache[id] = record
        return record
    }
}
```

Swift's `actor` gives the caching proxy safe concurrent access to its mutable cache without a manual lock.

## Pitfalls

- A caching proxy that never invalidates, silently serving stale data.
- Access-control proxies that can be bypassed because some code paths hold a reference to the real subject directly, not through the proxy.
- Retry proxies that retry non-idempotent operations, causing duplicate side effects.
- Remote/lazy proxies that hide latency or failure modes callers need to know about (a "simple getter" that can now block on the network or throw a new class of error).
- Proxy chains (cache → retry → auth → real) with unclear or undocumented ordering.

## See Also

- [decorator](decorator.md) — adding new behavior/data versus controlling access to existing behavior.
- [adapter](adapter.md) — translating an interface, versus preserving the same one.
- [circuit-breaker](circuit-breaker.md) — a specialized proxy that also tracks failure state and stops calling a failing dependency.
