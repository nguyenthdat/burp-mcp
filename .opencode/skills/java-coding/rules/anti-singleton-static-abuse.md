# anti-singleton-static-abuse

> Don't abuse the singleton pattern / static state

## Why It Matters

A singleton accessed through a static `getInstance()` call is a global variable wearing a design-pattern costume: every class that touches it becomes implicitly coupled to it, unit tests can't substitute a fake without reflection hacks or classloader tricks, and mutable static state on top of it makes concurrent tests (or concurrent requests) interfere with each other unpredictably.

## Bad

```java
public class ConfigManager {
  private static ConfigManager instance;
  private Map<String, String> settings = new HashMap<>();

  private ConfigManager() { }

  public static ConfigManager getInstance() {
    if (instance == null) {
      instance = new ConfigManager(); // Also not thread-safe under concurrent first access
    }
    return instance;
  }

  public void set(String key, String value) {
    settings.put(key, value);
  }

  public String get(String key) {
    return settings.get(key);
  }
}

// Any class can reach into global state from anywhere - impossible to
// test in isolation, and one test's mutation leaks into the next test
public class OrderService {
  public void process(Order order) {
    String currency = ConfigManager.getInstance().get("default.currency");
    // ...
  }
}
```

## Good

```java
// Plain, immutable, dependency-injected configuration
public final class AppConfig {
  private final Map<String, String> settings;

  public AppConfig(Map<String, String> settings) {
    this.settings = Map.copyOf(settings);
  }

  public String get(String key) {
    return settings.get(key);
  }
}

public class OrderService {
  private final AppConfig config;

  public OrderService(AppConfig config) { // Injected, swappable in tests
    this.config = config;
  }

  public void process(Order order) {
    String currency = config.get("default.currency");
    // ...
  }
}

// Tests inject a fresh, isolated instance - no shared global state
@Test
void usesConfiguredCurrency() {
  AppConfig testConfig = new AppConfig(Map.of("default.currency", "EUR"));
  OrderService service = new OrderService(testConfig);
  // ...
}
```

## See Also

- [`conc-holder-idiom-lazy-singleton`](conc-holder-idiom-lazy-singleton.md) - If a singleton is truly required, the thread-safe way to build one
- [`anti-overuse-of-static`](anti-overuse-of-static.md) - The broader anti-pattern of reaching for static instead of instances
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Dependency injection keeps collaborators' surfaces explicit and testable
