# adapter

> Make a foreign or legacy interface satisfy a contract your code already depends on.

## Intent & Pressure

Reach for Adapter when you must integrate a third-party library, legacy module, or external service whose interface doesn't match the abstraction your code depends on, and you cannot (or shouldn't) change either side. The pressure is a mismatched contract: different method names, different error conventions, different data shapes, a synchronous API where you need an async one, or vice versa.

Do not reach for it when you own both sides — just change one interface to match the other. Do not use Adapter to paper over a genuinely bad third-party API forever; note the wrapped complexity and keep the adapter thin enough that it's obvious what's being translated.

## Native-Construct Alternative

A single conversion function (`fn adapt(external: Foreign) -> Local`) is often enough when the mismatch is a one-time data transform. Promote to a wrapper type implementing a shared interface/trait when the local contract has multiple methods and callers need to hold the adapted object polymorphically.

## Language Implementations

### Rust

```rust
// Local contract
trait Logger {
    fn log(&self, level: Level, message: &str);
}

// Foreign library type, e.g. `tracing` or a vendor SDK, out of your control
struct LegacyLogWriter;
impl LegacyLogWriter {
    fn write_line(&self, line: &str) { /* ... */ }
}

struct LegacyLoggerAdapter(LegacyLogWriter);

impl Logger for LegacyLoggerAdapter {
    fn log(&self, level: Level, message: &str) {
        self.0.write_line(&format!("[{level:?}] {message}"));
    }
}
```

A newtype wrapper implementing the local trait respects Rust's orphan rules (you can't `impl Logger for LegacyLogWriter` if you own neither) and keeps translation in one place.

### TypeScript

```typescript
interface Logger {
  log(level: LogLevel, message: string): void;
}

// third-party class with an incompatible shape
class VendorLogger {
  writeLine(line: string): void { /* ... */ }
}

class VendorLoggerAdapter implements Logger {
  constructor(private readonly vendor: VendorLogger) {}
  log(level: LogLevel, message: string): void {
    this.vendor.writeLine(`[${LogLevel[level]}] ${message}`);
  }
}
```

### Python

```python
from typing import Protocol

class Logger(Protocol):
    def log(self, level: str, message: str) -> None: ...

class VendorLogger:  # third-party, incompatible signature
    def write_line(self, line: str) -> None: ...

class VendorLoggerAdapter:
    def __init__(self, vendor: VendorLogger) -> None:
        self._vendor = vendor

    def log(self, level: str, message: str) -> None:
        self._vendor.write_line(f"[{level}] {message}")
```

### Go

```go
type Logger interface {
    Log(level Level, message string)
}

type VendorLogger struct{} // third-party type
func (VendorLogger) WriteLine(line string) { /* ... */ }

type vendorLoggerAdapter struct{ vendor VendorLogger }

func (a vendorLoggerAdapter) Log(level Level, message string) {
    a.vendor.WriteLine(fmt.Sprintf("[%v] %s", level, message))
}

func NewLogger(v VendorLogger) Logger { return vendorLoggerAdapter{vendor: v} }
```

Go's implicit interfaces mean many "adapters" are free — if the method set already matches, no wrapper is needed at all. Write one only when names or signatures genuinely differ.

### C#

```csharp
public interface ILogger
{
    void Log(LogLevel level, string message);
}

public sealed class VendorLoggerAdapter : ILogger
{
    private readonly VendorLogger _vendor;
    public VendorLoggerAdapter(VendorLogger vendor) => _vendor = vendor;

    public void Log(LogLevel level, string message) =>
        _vendor.WriteLine($"[{level}] {message}");
}
```

### Kotlin

```kotlin
interface Logger {
    fun log(level: LogLevel, message: String)
}

class VendorLoggerAdapter(private val vendor: VendorLogger) : Logger {
    override fun log(level: LogLevel, message: String) {
        vendor.writeLine("[$level] $message")
    }
}
```

### C

```c
/* Local contract: struct of function pointers */
typedef struct logger {
    void (*log)(struct logger *self, int level, const char *message);
    void *state;
} logger_t;

/* Foreign library: vendor_log_writer_t with vendor_write_line(...) */
static void vendor_adapter_log(logger_t *self, int level, const char *message) {
    vendor_log_writer_t *vendor = (vendor_log_writer_t *)self->state;
    char line[256];
    snprintf(line, sizeof(line), "[%d] %s", level, message);
    vendor_write_line(vendor, line);
}

logger_t make_vendor_logger_adapter(vendor_log_writer_t *vendor) {
    return (logger_t){ .log = vendor_adapter_log, .state = vendor };
}
```

### C++

```cpp
class Logger {
public:
    virtual ~Logger() = default;
    virtual void log(LogLevel level, std::string_view message) = 0;
};

class VendorLoggerAdapter : public Logger {
public:
    explicit VendorLoggerAdapter(VendorLogger &vendor) : vendor_(vendor) {}
    void log(LogLevel level, std::string_view message) override {
        vendor_.writeLine(std::format("[{}] {}", static_cast<int>(level), message));
    }
private:
    VendorLogger &vendor_; // adapter does not own the wrapped object
};
```

### Swift

```swift
protocol Logger {
    func log(level: LogLevel, message: String)
}

// third-party class with an incompatible API
final class VendorLoggerAdapter: Logger {
    private let vendor: VendorLogger
    init(vendor: VendorLogger) { self.vendor = vendor }

    func log(level: LogLevel, message: String) {
        vendor.writeLine("[\(level)] \(message)")
    }
}
```

## Pitfalls

- Losing error information in translation (swallowing a foreign exception/error code instead of mapping it to a local typed error).
- Adapting a synchronous API to look asynchronous (or vice versa) without documenting that the underlying call still blocks/still can't be cancelled.
- Layering adapter upon adapter until nobody can trace which one does the real translation.
- Adapting so much surface area that the "adapter" becomes a second, divergent implementation of the local contract — if you're rewriting most of the behavior, it's not adaptation anymore.
- In Rust, trying to `impl` a foreign trait for a foreign type (orphan rule violation) instead of wrapping in a local newtype.

## See Also

- [facade](facade.md) — simplifying a subsystem's own interface, rather than translating to match a different one.
- [bridge](bridge.md) — designed-in variation between abstraction and implementation, versus retrofitting compatibility.
- [proxy](proxy.md) — same interface on both sides, adding policy rather than translating it.
