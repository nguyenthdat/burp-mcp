# observer

> Let multiple dependents react to a subject's state changes over time, without the subject knowing anything about them beyond a subscription contract.

## Intent & Pressure

Reach for Observer when a subject's state changes must trigger reactions in an open, dynamic set of dependents (UI reacting to model changes, cache invalidation listeners, metrics collectors) and that set of listeners can grow, shrink, or vary per deployment. The pressure is decoupling: the subject shouldn't need to know the concrete types or count of things reacting to it, and listeners should be addable/removable independently.

Do not reach for it when there's exactly one consumer of an event — pass a direct callback instead. Do not use it for cross-process communication — see [pub-sub](pub-sub.md) for that; Observer is the in-process, synchronous-or-lightly-async version.

## Native-Construct Alternative

A direct callback parameter (`fn(on_change: impl FnMut(&Event))`) is sufficient for exactly one consumer. Promote to Observer once multiple independent consumers must subscribe/unsubscribe over the subject's lifetime.

## Language Implementations

### Rust

```rust
trait Observer {
    fn on_event(&self, event: &Event);
}

struct Subject {
    observers: Vec<Weak<dyn Observer>>, // Weak: subject does not keep observers alive
}

impl Subject {
    fn subscribe(&mut self, observer: Weak<dyn Observer>) {
        self.observers.push(observer);
    }
    fn notify(&self, event: &Event) {
        for observer in &self.observers {
            if let Some(observer) = observer.upgrade() {
                observer.on_event(event);
            }
        }
    }
}
```

Async fan-out to many tasks typically uses a `tokio::sync::broadcast` channel instead of a callback list.

### TypeScript

```typescript
type Listener = (event: Event) => void;

class Subject {
  private listeners = new Set<Listener>();

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener); // unsubscribe handle
  }

  notify(event: Event): void {
    for (const listener of this.listeners) listener(event);
  }
}
```

Returning an unsubscribe function from `subscribe` avoids the classic "forgot to remove listener" leak.

### Python

```python
class Subject:
    def __init__(self) -> None:
        self._listeners: list[Callable[[Event], None]] = []

    def subscribe(self, listener: Callable[[Event], None]) -> Callable[[], None]:
        self._listeners.append(listener)
        def unsubscribe() -> None:
            self._listeners.remove(listener)
        return unsubscribe

    def notify(self, event: Event) -> None:
        for listener in list(self._listeners):  # copy: tolerate mutation during iteration
            listener(event)
```

### Go

```go
type Subject struct {
    mu        sync.Mutex
    listeners map[int]func(Event)
    nextID    int
}

func (s *Subject) Subscribe(listener func(Event)) (unsubscribe func()) {
    s.mu.Lock()
    id := s.nextID
    s.nextID++
    s.listeners[id] = listener
    s.mu.Unlock()
    return func() {
        s.mu.Lock()
        delete(s.listeners, id)
        s.mu.Unlock()
    }
}

func (s *Subject) Notify(event Event) {
    s.mu.Lock()
    listeners := make([]func(Event), 0, len(s.listeners))
    for _, l := range s.listeners {
        listeners = append(listeners, l)
    }
    s.mu.Unlock()
    for _, l := range listeners {
        l(event)
    }
}
```

Copy the listener slice under the lock before invoking callbacks outside it — never hold the mutex while calling into unknown listener code.

### C#

```csharp
public sealed class Subject
{
    public event EventHandler<Event>? Changed;

    public void Notify(Event evt) => Changed?.Invoke(this, evt);
}

// usage: subject.Changed += (sender, evt) => { ... };
//        subject.Changed -= handler; // explicit unsubscribe required to avoid leaks
```

C#'s built-in `event` keyword is the idiomatic Observer; remember `-=` to unsubscribe, especially for long-lived subjects.

### Kotlin

```kotlin
class Subject {
    private val _events = MutableSharedFlow<Event>()
    val events: SharedFlow<Event> = _events

    suspend fun notify(event: Event) = _events.emit(event)
}

// usage: scope.launch { subject.events.collect { event -> ... } }
```

`SharedFlow`/`StateFlow` is the idiomatic Kotlin Observer for coroutine-based code; subscription lifetime is tied to the collecting coroutine's scope.

### C

```c
typedef void (*observer_fn)(void *ctx, const event_t *event);

typedef struct subject {
    struct { observer_fn fn; void *ctx; } observers[MAX_OBSERVERS];
    size_t count;
} subject_t;

int subject_subscribe(subject_t *s, observer_fn fn, void *ctx) {
    if (s->count >= MAX_OBSERVERS) return -1;
    s->observers[s->count].fn = fn;
    s->observers[s->count].ctx = ctx;
    s->count++;
    return 0;
}

void subject_notify(subject_t *s, const event_t *event) {
    for (size_t i = 0; i < s->count; i++) {
        s->observers[i].fn(s->observers[i].ctx, event);
    }
}
```

A fixed-size array of function-pointer+context pairs is the common C pattern; document that unsubscribe requires compacting the array (and how concurrent notify/unsubscribe is handled, if any).

### C++

```cpp
class Subject {
public:
    using Listener = std::function<void(const Event &)>;
    using SubscriptionId = std::size_t;

    SubscriptionId subscribe(Listener listener) {
        auto id = nextId_++;
        listeners_[id] = std::move(listener);
        return id;
    }
    void unsubscribe(SubscriptionId id) { listeners_.erase(id); }

    void notify(const Event &event) {
        auto snapshot = listeners_; // copy to tolerate mutation during iteration
        for (auto &[id, listener] : snapshot) listener(event);
    }
private:
    std::unordered_map<SubscriptionId, Listener> listeners_;
    SubscriptionId nextId_ = 0;
};
```

### Swift

```swift
final class Subject {
    private var listeners: [UUID: (Event) -> Void] = [:]

    @discardableResult
    func subscribe(_ listener: @escaping (Event) -> Void) -> UUID {
        let id = UUID()
        listeners[id] = listener
        return id
    }
    func unsubscribe(_ id: UUID) { listeners.removeValue(forKey: id) }

    func notify(_ event: Event) {
        for listener in listeners.values { listener(event) }
    }
}
```

For SwiftUI/Combine code, prefer `@Published`/`PassthroughSubject` over a hand-rolled subject; they already manage subscription lifetime correctly.

## Pitfalls

- Subjects holding strong references to observers, keeping them (and everything they retain) alive forever — use `Weak`/`weak` or an explicit unsubscribe contract.
- Mutating the listener collection while iterating it during `notify` (a listener that subscribes/unsubscribes from inside its own callback) causing a crash or skipped notification.
- Calling listener callbacks while holding a lock, risking deadlock if a listener re-enters the subject.
- No backpressure/ordering guarantee documented for async delivery, causing silent event drops under load.
- Treating "notify" as fire-and-forget when a listener's failure should actually propagate or be logged.

## See Also

- [mediator](mediator.md) — centralized coordination with a known owner, versus decoupled broadcast.
- [pub-sub](pub-sub.md) — the cross-process/distributed generalization of Observer.
- [state](state.md) — often the mechanism that decides when a subject should notify observers (on transition).
