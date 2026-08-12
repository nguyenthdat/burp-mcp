# null-object

> Replace scattered null/None/undefined checks with a neutral, do-nothing implementation of the same interface.

## Intent & Pressure

Reach for Null Object when callers repeatedly check for absence before doing anything useful with a value (`if (logger != null) logger.log(...)` at every call site), and a "do nothing, safely" implementation of the same interface would eliminate that repeated branching entirely. The pressure is defensive-check duplication scattered across a codebase for a case that's usually genuinely fine to no-op.

Do not reach for it when absence needs to be handled differently at each call site (some callers must throw, some must default, some must prompt) — collapsing that distinction into a single silent no-op removes information callers need. Most languages' own `Option`/`Optional`/nullable-with-null-coalescing already solve simple "value or default" cases better than a bespoke Null Object class.

## Native-Construct Alternative

An `Option`/optional type with combinators (`unwrap_or`, `?? defaultValue`, `.getOrElse`, `or_else`) handles most "value or default" cases directly. Reach for a full Null Object implementing the real interface specifically when the "default" behavior is itself non-trivial (e.g., a `NullLogger` that must implement several methods consistently, not just supply a fallback scalar).

## Language Implementations

### Rust

```rust
trait Logger {
    fn log(&self, message: &str);
}

struct NullLogger;
impl Logger for NullLogger {
    fn log(&self, _message: &str) {} // intentionally does nothing
}

fn process(logger: &dyn Logger) {
    logger.log("processing started"); // no null check needed at any call site
}
```

Rust often prefers `Option<&dyn Logger>` with `.map(|l| l.log(...))`, but a `NullLogger` is clearer when `Logger` has many methods that would otherwise need repeated `if let Some`.

### TypeScript

```typescript
interface Logger {
  log(message: string): void;
}

class NullLogger implements Logger {
  log(_message: string): void {} // no-op
}

function process(logger: Logger = new NullLogger()): void {
  logger.log("processing started");
}
```

### Python

```python
class Logger(Protocol):
    def log(self, message: str) -> None: ...

class NullLogger:
    def log(self, message: str) -> None:
        pass  # intentionally does nothing

def process(logger: Logger = NullLogger()) -> None:
    logger.log("processing started")
```

### Go

```go
type Logger interface {
    Log(message string)
}

type nullLogger struct{}
func (nullLogger) Log(message string) {} // no-op

func Process(logger Logger) {
    if logger == nil {
        logger = nullLogger{}
    }
    logger.Log("processing started")
}
```

### C#

```csharp
public interface ILogger
{
    void Log(string message);
}

public sealed class NullLogger : ILogger
{
    public static readonly NullLogger Instance = new();
    public void Log(string message) { } // no-op
}

public void Process(ILogger? logger = null)
{
    (logger ?? NullLogger.Instance).Log("processing started");
}
```

### Kotlin

```kotlin
interface Logger {
    fun log(message: String)
}

object NullLogger : Logger {
    override fun log(message: String) {} // no-op
}

fun process(logger: Logger = NullLogger) {
    logger.log("processing started")
}
```

### C

```c
typedef struct logger {
    void (*log)(struct logger *self, const char *message);
} logger_t;

static void null_log(logger_t *self, const char *message) { (void)self; (void)message; }
static logger_t null_logger = { .log = null_log };

void process(logger_t *logger) {
    if (!logger) logger = &null_logger;
    logger->log(logger, "processing started");
}
```

### C++

```cpp
class Logger {
public:
    virtual ~Logger() = default;
    virtual void log(std::string_view message) = 0;
};

class NullLogger : public Logger {
public:
    void log(std::string_view) override {} // no-op
};

void process(Logger &logger = *NullLoggerInstance()) {
    logger.log("processing started");
}
```

### Swift

```swift
protocol Logger {
    func log(_ message: String)
}

struct NullLogger: Logger {
    func log(_ message: String) {} // no-op
}

func process(logger: Logger = NullLogger()) {
    logger.log("processing started")
}
```

## Pitfalls

- Using Null Object where callers actually need to distinguish "absent" from "present but no-op" — that information gets silently lost.
- Hiding a real configuration bug (a logger that should have been wired up) behind a silently-succeeding no-op, delaying detection.
- A Null Object that isn't actually side-effect-free (e.g., a `NullCache` that still allocates) misleading callers about its cost.
- Overusing Null Object instead of a plain `Option`/nullable type when the "default behavior" is just a scalar fallback, not multi-method behavior.

## See Also

- [strategy](strategy.md) — Null Object is a specific, no-op Strategy implementation.
- [proxy](proxy.md) — both provide a stand-in implementing the same interface, but Proxy adds behavior/policy rather than removing it.
- [specification](specification.md) — a similarly small, composable pattern for expressing "always true"/"always false" predicates.
