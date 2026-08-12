# circuit-breaker

> Stop calling a dependency that's failing repeatedly, so the caller fails fast and the dependency gets a chance to recover, instead of every caller piling on more load.

## Intent & Pressure

Reach for Circuit Breaker when a downstream dependency (a remote service, a database, a third-party API) can fail intermittently in a way that repeated retries make worse — cascading failure, thread/connection pool exhaustion, retry storms. The pressure is protecting both the caller (fail fast instead of hanging/timing out repeatedly) and the callee (stop adding load to something already struggling), with an automatic path back to normal once the dependency recovers.

Do not reach for it when failures are rare and a simple timeout plus bounded retry-with-backoff already handles them — Circuit Breaker adds state machine complexity (closed/open/half-open, failure thresholds, recovery timers) that's only worth it once a dependency's failures are common/bursty enough to threaten cascading failure.

## Native-Construct Alternative

A timeout with retry-and-exponential-backoff on individual calls handles occasional, independent failures. Escalate to a stateful Circuit Breaker specifically when failures cluster (an outage) and you need to stop hammering the dependency across many callers/threads until it recovers.

## Language Implementations

### Rust

```rust
enum CircuitState { Closed, Open { opened_at: Instant }, HalfOpen }

struct CircuitBreaker {
    state: Mutex<CircuitState>,
    failure_count: AtomicU32,
    threshold: u32,
    reset_after: Duration,
}

impl CircuitBreaker {
    async fn call<F, Fut, T>(&self, f: F) -> Result<T, CircuitError>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<T, CallError>> {
        if let CircuitState::Open { opened_at } = *self.state.lock().unwrap() {
            if opened_at.elapsed() < self.reset_after {
                return Err(CircuitError::Open);
            }
            *self.state.lock().unwrap() = CircuitState::HalfOpen;
        }
        match f().await {
            Ok(value) => { self.on_success(); Ok(value) }
            Err(err) => { self.on_failure(); Err(CircuitError::Call(err)) }
        }
    }
}
```

### TypeScript

```typescript
type CircuitState = { kind: "closed" } | { kind: "open"; openedAt: number } | { kind: "halfOpen" };

class CircuitBreaker {
  private state: CircuitState = { kind: "closed" };
  private failures = 0;
  constructor(private threshold: number, private resetAfterMs: number) {}

  async call<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state.kind === "open") {
      if (Date.now() - this.state.openedAt < this.resetAfterMs) throw new Error("circuit open");
      this.state = { kind: "halfOpen" };
    }
    try {
      const result = await fn();
      this.failures = 0;
      this.state = { kind: "closed" };
      return result;
    } catch (err) {
      if (++this.failures >= this.threshold) this.state = { kind: "open", openedAt: Date.now() };
      throw err;
    }
  }
}
```

### Python

```python
import time
from enum import Enum, auto

class State(Enum):
    CLOSED = auto()
    OPEN = auto()
    HALF_OPEN = auto()

class CircuitBreaker:
    def __init__(self, threshold: int, reset_after: float) -> None:
        self._state = State.CLOSED
        self._failures = 0
        self._opened_at = 0.0
        self._threshold = threshold
        self._reset_after = reset_after

    def call(self, fn):
        if self._state is State.OPEN:
            if time.monotonic() - self._opened_at < self._reset_after:
                raise CircuitOpenError()
            self._state = State.HALF_OPEN
        try:
            result = fn()
        except Exception:
            self._failures += 1
            if self._failures >= self._threshold:
                self._state = State.OPEN
                self._opened_at = time.monotonic()
            raise
        else:
            self._failures = 0
            self._state = State.CLOSED
            return result
```

### Go

```go
type CircuitBreaker struct {
    mu         sync.Mutex
    state      string // "closed", "open", "half-open"
    failures   int
    openedAt   time.Time
    threshold  int
    resetAfter time.Duration
}

func (cb *CircuitBreaker) Call(fn func() error) error {
    cb.mu.Lock()
    if cb.state == "open" {
        if time.Since(cb.openedAt) < cb.resetAfter {
            cb.mu.Unlock()
            return ErrCircuitOpen
        }
        cb.state = "half-open"
    }
    cb.mu.Unlock()

    err := fn()
    cb.mu.Lock()
    defer cb.mu.Unlock()
    if err != nil {
        cb.failures++
        if cb.failures >= cb.threshold {
            cb.state = "open"
            cb.openedAt = time.Now()
        }
        return err
    }
    cb.failures = 0
    cb.state = "closed"
    return nil
}
```

### C#

```csharp
public sealed class CircuitBreaker
{
    private CircuitState _state = CircuitState.Closed;
    private int _failures;
    private DateTimeOffset _openedAt;
    private readonly int _threshold;
    private readonly TimeSpan _resetAfter;

    public async Task<T> CallAsync<T>(Func<Task<T>> fn)
    {
        if (_state == CircuitState.Open)
        {
            if (DateTimeOffset.UtcNow - _openedAt < _resetAfter) throw new CircuitOpenException();
            _state = CircuitState.HalfOpen;
        }
        try
        {
            var result = await fn();
            _failures = 0;
            _state = CircuitState.Closed;
            return result;
        }
        catch
        {
            if (++_failures >= _threshold) { _state = CircuitState.Open; _openedAt = DateTimeOffset.UtcNow; }
            throw;
        }
    }
}
```

Production .NET code typically uses Polly's `CircuitBreakerPolicy` instead of a hand-rolled implementation.

### Kotlin

```kotlin
enum class CircuitState { CLOSED, OPEN, HALF_OPEN }

class CircuitBreaker(private val threshold: Int, private val resetAfter: Duration) {
    private var state = CircuitState.CLOSED
    private var failures = 0
    private var openedAt: Instant? = null

    suspend fun <T> call(fn: suspend () -> T): T {
        if (state == CircuitState.OPEN) {
            if (Duration.between(openedAt, Instant.now()) < resetAfter) throw CircuitOpenException()
            state = CircuitState.HALF_OPEN
        }
        return try {
            val result = fn()
            failures = 0
            state = CircuitState.CLOSED
            result
        } catch (e: Exception) {
            if (++failures >= threshold) { state = CircuitState.OPEN; openedAt = Instant.now() }
            throw e
        }
    }
}
```

### C

```c
typedef enum { CB_CLOSED, CB_OPEN, CB_HALF_OPEN } cb_state_t;

typedef struct circuit_breaker {
    cb_state_t state;
    int failures, threshold;
    time_t opened_at;
    double reset_after_secs;
} circuit_breaker_t;

int cb_call(circuit_breaker_t *cb, int (*fn)(void *), void *ctx) {
    if (cb->state == CB_OPEN) {
        if (difftime(time(NULL), cb->opened_at) < cb->reset_after_secs) return -1; /* open */
        cb->state = CB_HALF_OPEN;
    }
    int rc = fn(ctx);
    if (rc != 0) {
        cb->failures++;
        if (cb->failures >= cb->threshold) { cb->state = CB_OPEN; cb->opened_at = time(NULL); }
        return rc;
    }
    cb->failures = 0;
    cb->state = CB_CLOSED;
    return 0;
}
```

### C++

```cpp
enum class CircuitState { Closed, Open, HalfOpen };

class CircuitBreaker {
public:
    CircuitBreaker(int threshold, std::chrono::milliseconds resetAfter)
        : threshold_(threshold), resetAfter_(resetAfter) {}

    template <typename Fn>
    auto call(Fn fn) -> decltype(fn()) {
        if (state_ == CircuitState::Open) {
            if (std::chrono::steady_clock::now() - openedAt_ < resetAfter_) throw CircuitOpenError();
            state_ = CircuitState::HalfOpen;
        }
        try {
            auto result = fn();
            failures_ = 0;
            state_ = CircuitState::Closed;
            return result;
        } catch (...) {
            if (++failures_ >= threshold_) { state_ = CircuitState::Open; openedAt_ = std::chrono::steady_clock::now(); }
            throw;
        }
    }
private:
    CircuitState state_ = CircuitState::Closed;
    int failures_ = 0, threshold_;
    std::chrono::steady_clock::time_point openedAt_;
    std::chrono::milliseconds resetAfter_;
};
```

### Swift

```swift
enum CircuitState { case closed, open(openedAt: Date), halfOpen }

actor CircuitBreaker {
    private var state: CircuitState = .closed
    private var failures = 0
    private let threshold: Int
    private let resetAfter: TimeInterval

    init(threshold: Int, resetAfter: TimeInterval) {
        self.threshold = threshold
        self.resetAfter = resetAfter
    }

    func call<T>(_ fn: () async throws -> T) async throws -> T {
        if case .open(let openedAt) = state {
            guard Date().timeIntervalSince(openedAt) >= resetAfter else { throw CircuitOpenError() }
            state = .halfOpen
        }
        do {
            let result = try await fn()
            failures = 0
            state = .closed
            return result
        } catch {
            failures += 1
            if failures >= threshold { state = .open(openedAt: Date()) }
            throw error
        }
    }
}
```

Swift's `actor` gives the breaker's mutable state safe concurrent access without a manual lock.

## Pitfalls

- Reinventing this instead of using a well-tested library (Polly for .NET, resilience4j for JVM languages, `failsafe`/tower middleware for others) when one is available for the stack.
- No half-open trial limiting — letting every caller flood the dependency the instant the reset timer expires, re-tripping the breaker immediately.
- Sharing one breaker instance across unrelated dependencies, so one failing call trips the circuit for calls that were fine.
- No distinction between the failures that should trip the breaker (timeouts, 5xx) and client errors that shouldn't (4xx, validation failures).
- No metrics/alerting on state transitions, so an open circuit silently degrades functionality without anyone noticing.

## See Also

- [proxy](proxy.md) — Circuit Breaker is a specialized stateful proxy around a specific dependency call.
- [state](state.md) — the closed/open/half-open transitions are themselves a small state machine worth modeling explicitly.
- [pipeline-middleware](pipeline-middleware.md) — a circuit breaker is commonly implemented as one middleware stage among several.
