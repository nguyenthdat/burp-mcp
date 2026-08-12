# object-pool

> Reuse a fixed set of expensive-to-create objects instead of allocating and tearing them down repeatedly.

## Intent & Pressure

Reach for Object Pool when construction or teardown of an object is measurably expensive relative to its use (DB/network connections, thread pools, large fixed-size buffers, GPU/OS handles), and the workload creates/discards many short-lived instances under load. The pressure is a proven allocation or setup/teardown cost, not a guess — profile first.

Do not reach for it when the runtime's allocator or garbage collector already makes plain allocation cheap (most short-lived, small objects in GC languages). Object pooling adds real complexity — checkout/return discipline, leak detection, poisoned-object cleanup, and shrinking capacity — that is only worth it when construction cost dominates.

## Native-Construct Alternative

Just allocate normally. Most languages' allocators (and connection libraries, e.g., database drivers) already provide pooling internally for the common expensive resources (DB connections, thread pools) — reuse that instead of writing a bespoke pool. Only write your own pool for a resource with no existing pooled library and a demonstrated cost.

## Language Implementations

### Rust

```rust
pub struct Pool<T> {
    items: Mutex<Vec<T>>,
    create: fn() -> T,
}

impl<T> Pool<T> {
    pub fn new(create: fn() -> T) -> Self {
        Self { items: Mutex::new(Vec::new()), create }
    }
    pub fn acquire(&self) -> PooledItem<'_, T> {
        let item = self.items.lock().unwrap().pop().unwrap_or_else(|| (self.create)());
        PooledItem { item: Some(item), pool: self }
    }
    fn release(&self, item: T) {
        self.items.lock().unwrap().push(item);
    }
}

pub struct PooledItem<'a, T> {
    item: Option<T>,
    pool: &'a Pool<T>,
}

impl<T> Drop for PooledItem<'_, T> {
    fn drop(&mut self) {
        if let Some(item) = self.item.take() {
            self.pool.release(item);
        }
    }
}
```

`Drop` returns the item to the pool automatically — RAII makes the checkout/return discipline impossible to forget.

### TypeScript

```typescript
class Pool<T> {
  private items: T[] = [];
  constructor(private readonly create: () => T, private readonly reset?: (item: T) => void) {}

  acquire(): T {
    return this.items.pop() ?? this.create();
  }
  release(item: T): void {
    this.reset?.(item);
    this.items.push(item);
  }
}

// usage: caller must remember to release, or wrap in a try/finally helper
const pool = new Pool(() => new ExpensiveBuffer());
const buf = pool.acquire();
try {
  use(buf);
} finally {
  pool.release(buf);
}
```

### Python

```python
from contextlib import contextmanager

class Pool:
    def __init__(self, create):
        self._create = create
        self._items: list = []

    @contextmanager
    def acquire(self):
        item = self._items.pop() if self._items else self._create()
        try:
            yield item
        finally:
            self._items.append(item)

pool = Pool(create=ExpensiveBuffer)
with pool.acquire() as buf:
    use(buf)
```

A context manager makes checkout/return automatic even on exceptions — the Pythonic equivalent of Rust's `Drop`.

### Go

```go
type Pool[T any] struct {
    mu    sync.Mutex
    items []T
    New   func() T
}

func (p *Pool[T]) Acquire() T {
    p.mu.Lock()
    defer p.mu.Unlock()
    if n := len(p.items); n > 0 {
        item := p.items[n-1]
        p.items = p.items[:n-1]
        return item
    }
    return p.New()
}

func (p *Pool[T]) Release(item T) {
    p.mu.Lock()
    defer p.mu.Unlock()
    p.items = append(p.items, item)
}
```

Prefer the standard library's `sync.Pool` for GC-pressure reduction on short-lived temporary objects — it is explicitly allowed to drop items under memory pressure, so never store the only reference to something in it.

### C#

```csharp
public sealed class ObjectPool<T>
{
    private readonly ConcurrentBag<T> _items = new();
    private readonly Func<T> _create;

    public ObjectPool(Func<T> create) => _create = create;

    public T Rent() => _items.TryTake(out var item) ? item : _create();
    public void Return(T item) => _items.Add(item);
}
```

Prefer `Microsoft.Extensions.ObjectPool` (`ObjectPool<T>`, `DefaultObjectPoolProvider`) in production ASP.NET code instead of hand-rolling one.

### Kotlin

```kotlin
class Pool<T>(private val create: () -> T) {
    private val items = ArrayDeque<T>()
    private val lock = Any()

    fun acquire(): T = synchronized(lock) { items.removeLastOrNull() } ?: create()
    fun release(item: T) = synchronized(lock) { items.addLast(item) }
}

inline fun <T, R> Pool<T>.use(block: (T) -> R): R {
    val item = acquire()
    try {
        return block(item)
    } finally {
        release(item)
    }
}
```

### C

```c
typedef struct buffer_pool {
    buffer_t *items[POOL_CAPACITY];
    size_t    count;
    pthread_mutex_t lock;
} buffer_pool_t;

buffer_t *pool_acquire(buffer_pool_t *pool) {
    pthread_mutex_lock(&pool->lock);
    buffer_t *item = pool->count > 0 ? pool->items[--pool->count] : buffer_create();
    pthread_mutex_unlock(&pool->lock);
    return item;
}

void pool_release(buffer_pool_t *pool, buffer_t *item) {
    pthread_mutex_lock(&pool->lock);
    if (pool->count < POOL_CAPACITY) {
        pool->items[pool->count++] = item;
    } else {
        buffer_destroy(item); /* pool full: free instead of leaking */
    }
    pthread_mutex_unlock(&pool->lock);
}
```

C needs an explicit fallback for a full pool (free the item) since there is no GC to reclaim an unpooled excess item.

### C++

```cpp
template <typename T>
class ObjectPool {
public:
    explicit ObjectPool(std::function<T()> create) : create_(std::move(create)) {}

    struct Deleter {
        ObjectPool *pool;
        void operator()(T *item) const { pool->release(std::unique_ptr<T>(item)); }
    };
    using Handle = std::unique_ptr<T, Deleter>;

    Handle acquire() {
        std::lock_guard lock(mutex_);
        if (!items_.empty()) {
            auto item = std::move(items_.back());
            items_.pop_back();
            return Handle(item.release(), Deleter{this});
        }
        return Handle(new T(create_()), Deleter{this});
    }

private:
    void release(std::unique_ptr<T> item) {
        std::lock_guard lock(mutex_);
        items_.push_back(std::move(item));
    }
    std::function<T()> create_;
    std::vector<std::unique_ptr<T>> items_;
    std::mutex mutex_;
};
```

A custom deleter on `unique_ptr` gives RAII-style automatic return-to-pool, matching Rust's `Drop` approach.

### Swift

```swift
final class Pool<T> {
    private var items: [T] = []
    private let create: () -> T
    private let lock = NSLock()

    init(create: @escaping () -> T) { self.create = create }

    func acquire() -> T {
        lock.lock(); defer { lock.unlock() }
        return items.popLast() ?? create()
    }
    func release(_ item: T) {
        lock.lock(); defer { lock.unlock() }
        items.append(item)
    }
}
```

## Pitfalls

- Pooling objects that hold onto stale, sensitive, or connection-specific state without resetting it before reuse (a classic source of cross-request data leaks).
- Pooling small, cheap-to-allocate objects "for performance" without a profile showing a real cost — the pool's locking/bookkeeping can be slower than just allocating.
- Losing a checked-out item on an early return/exception, leaking pool capacity — always release via RAII/`finally`/`defer`/context manager, not manual bookkeeping.
- Growing the pool unbounded under load instead of capping size and falling back to plain allocation or backpressure.
- Sharing a pool across threads/goroutines without synchronizing acquire/release.

## See Also

- [singleton](singleton.md) — do not conflate "one shared pool" with "one shared instance"; the pool itself may be a singleton, but the pooled items are not.
- [flyweight](flyweight.md) — sharing immutable data versus reusing mutable, exclusively-owned instances.
- [prototype](prototype.md) — an alternative when copying is cheaper than pooling.
