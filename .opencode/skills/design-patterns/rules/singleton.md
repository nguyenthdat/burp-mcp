# singleton

> Guarantee a single process-wide instance and identity — and, in almost every real case, prefer not to.

## Intent & Pressure

Singleton exists for resources that genuinely have one identity per process: a logging sink, a hardware handle, a metrics registry that must aggregate globally. The pressure that justifies it is a real, provable constraint that a second instance would be incorrect (not merely "expensive to create" — that's Object Pool or Flyweight, not Singleton).

Do not reach for Singleton as a default way to share configuration, a database handle, or a cache — those should be constructed once at the composition root and passed in (see [dependency-injection](dependency-injection.md)). Global mutable Singletons are the single most common source of hidden coupling and un-testable code across every language: any test that runs after another test that touched the singleton inherits its state.

## Native-Construct Alternative

Prefer constructing the shared instance once in `main`/the composition root/the DI container and passing it down explicitly. If a language-level one-time-init primitive is unavoidable (a truly process-global immutable resource), use it only for that — never as a substitute for passing a dependency, and never for mutable state that tests need to reset.

## Language Implementations

### Rust

```rust
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config::load().expect("config must load"))
}
```

`OnceLock`/`LazyLock` are for one-time **immutable** initialization only — never `static mut`. If mutation is required, wrap the inner value in a `Mutex`/`RwLock`, and still prefer passing the `Arc<T>` explicitly over reading a `static` from deep in the call stack. Tests needing a fresh instance must inject a fresh value rather than relying on process-wide reset.

### TypeScript

```typescript
export class ConfigRegistry {
  private static instance: ConfigRegistry | undefined;

  private constructor(private readonly values: Record<string, string>) {}

  static getInstance(): ConfigRegistry {
    if (!ConfigRegistry.instance) {
      ConfigRegistry.instance = new ConfigRegistry(loadConfig());
    }
    return ConfigRegistry.instance;
  }
}
```

Prefer a module-level constant created once at import time, or better, construct one instance in the app's entry point and pass it through constructors/hooks — avoid `getInstance()` call sites scattered through business logic, which are impossible to substitute in tests without module-mocking hacks.

### Python

```python
# module-level singleton: Python modules are already imported once
_config: Config | None = None

def get_config() -> Config:
    global _config
    if _config is None:
        _config = Config.load()
    return _config
```

Prefer passing `config` as a function/constructor argument. If a true singleton is required, a module-level lazily-initialized variable is idiomatic; avoid a metaclass-based "Singleton pattern" class — it mostly adds ceremony Python doesn't need.

### Go

```go
var (
    configOnce sync.Once
    config     *Config
)

func GetConfig() *Config {
    configOnce.Do(func() {
        config = loadConfig()
    })
    return config
}
```

`sync.Once` is the idiomatic one-time-init primitive. Still prefer constructing `*Config` once in `main` and passing it through struct fields/function parameters over a package-level accessor.

### C#

```csharp
public sealed class ConfigRegistry
{
    private static readonly Lazy<ConfigRegistry> Lazy = new(() => new ConfigRegistry(LoadConfig()));
    public static ConfigRegistry Instance => Lazy.Value;

    private ConfigRegistry(IReadOnlyDictionary<string, string> values) => Values = values;
    public IReadOnlyDictionary<string, string> Values { get; }
}
```

In ASP.NET Core and most modern C#, prefer registering the instance as a DI **singleton lifetime** service (`services.AddSingleton<IConfig>(...)`) instead of a static `Instance` property — the DI container still hands out one shared instance, but consumers depend on an injected interface, which stays testable.

### Kotlin

```kotlin
object ConfigRegistry {
    val values: Map<String, String> by lazy { loadConfig() }
}
```

Kotlin `object` is a language-level Singleton with thread-safe lazy initialization built in. Still prefer a DI-provided `@Singleton`-scoped class (Hilt/Koin/manual) over a top-level `object` for anything that needs a test double.

### C

```c
static config_t *g_config = NULL;
static pthread_once_t g_config_once = PTHREAD_ONCE_INIT;

static void config_init(void) {
    g_config = config_load();
}

const config_t *config_get(void) {
    pthread_once(&g_config_once, config_init);
    return g_config;
}
```

`pthread_once` is the standard one-time-init guard in C. Never use a plain global pointer without a guard in multi-threaded code, and document that `g_config` is never freed (or provide an explicit teardown function called once at shutdown, not relied on for resets between tests).

### C++

```cpp
class ConfigRegistry {
public:
    static ConfigRegistry &instance() {
        static ConfigRegistry instance; // magic static: thread-safe since C++11
        return instance;
    }
    ConfigRegistry(const ConfigRegistry &) = delete;
    ConfigRegistry &operator=(const ConfigRegistry &) = delete;

private:
    ConfigRegistry() = default;
};
```

The C++11 "magic static" guarantees thread-safe one-time initialization without manual locking. Still prefer passing `ConfigRegistry&`/a reference to the dependency through constructors over calling `instance()` from deep in business logic.

### Swift

```swift
final class ConfigRegistry {
    static let shared = ConfigRegistry(values: Self.loadConfig())

    let values: [String: String]
    private init(values: [String: String]) { self.values = values }
    private static func loadConfig() -> [String: String] { /* ... */ [:] }
}
```

`static let` is lazily and thread-safely initialized exactly once. As with the other languages, prefer injecting `ConfigRegistry` (or a protocol it conforms to) through initializers over reading `.shared` inside deeply nested types, specifically so tests can substitute a fake.

## Pitfalls

- Using Singleton to share *any* dependency by default instead of passing it explicitly — this is the single biggest source of hidden coupling flagged in review.
- Mutable global state with no synchronization in a multi-threaded/async context.
- Relying on a one-time-init primitive (`OnceLock`, `sync.Once`, `Lazy`, magic static) as a test-reset mechanism — it isn't one; inject fresh state or isolate processes instead.
- A Singleton that reaches out to other singletons at construction time, creating an undocumented, hard-to-order global initialization graph.
- Hiding a Singleton behind an innocuous-looking static/module function so callers don't realize they've taken a hidden dependency.

## See Also

- [dependency-injection](dependency-injection.md) — the preferred alternative for sharing one instance without hiding the dependency.
- [object-pool](object-pool.md) — when the real pressure is reuse cost, not identity.
- [flyweight](flyweight.md) — when the real pressure is shared immutable data, not a single mutable instance.
