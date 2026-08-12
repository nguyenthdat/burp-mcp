# fn-composition-over-inheritance

> Compose small functions instead of building class inheritance hierarchies

## Why It Matters

Class inheritance couples behavior to a rigid, single-parent hierarchy: adding a new combination of behaviors often forces either duplicated code across sibling classes or an awkward "god" base class that grows to accommodate every subclass's needs (the classic diamond and fragile-base-class problems). Composing small, independently-testable functions avoids that coupling entirely — each function has one job, can be tested in isolation, and combinations are just function calls, not new classes in a hierarchy that has to be planned in advance.

## Bad

```typescript
class Logger {
  log(msg: string) { console.log(msg); }
}

class ValidatingLogger extends Logger {
  log(msg: string) {
    if (msg.trim().length === 0) return;
    super.log(msg);
  }
}

class TimestampedValidatingLogger extends ValidatingLogger {
  log(msg: string) {
    super.log(`[${new Date().toISOString()}] ${msg}`);
  }
}

// Need "timestamped but not validating"? Or "validating but not timestamped"?
// You're stuck writing a new class for every combination.
```

## Good

```typescript
type LogFn = (msg: string) => void;

const baseLogger: LogFn = (msg) => console.log(msg);

function withValidation(log: LogFn): LogFn {
  return (msg) => {
    if (msg.trim().length === 0) return;
    log(msg);
  };
}

function withTimestamp(log: LogFn): LogFn {
  return (msg) => log(`[${new Date().toISOString()}] ${msg}`);
}

// Any combination, in any order, without a new class for each:
const logger = withTimestamp(withValidation(baseLogger));
const validatingOnly = withValidation(baseLogger);
const timestampedOnly = withTimestamp(baseLogger);
```

## Composing Object Behavior, Not Just Functions

The same idea applies to objects: build capability by injecting collaborators rather than extending a base class.

```typescript
interface Notifier {
  notify(message: string): Promise<void>;
}

class OrderService {
  constructor(private readonly notifier: Notifier) {}

  async placeOrder(order: Order) {
    // ... business logic ...
    await this.notifier.notify(`Order ${order.id} placed`);
  }
}

// Swap behavior by injecting a different Notifier, no subclassing required.
const service = new OrderService(new EmailNotifier());
const testService = new OrderService(new InMemoryNotifier());
```

## When Inheritance Still Fits

A shallow, single-level `extends` is fine for genuine "is-a" relationships with a stable shared contract (e.g., a small hierarchy of DOM-like `Error` subclasses, or a UI framework's base component class mandated by the framework itself). The anti-pattern is *deep* or *multi-purpose* hierarchies used to share behavior rather than to model a true taxonomy.

## See Also

- [fn-pure-functions](fn-pure-functions.md) - Prefer pure functions with no hidden side effects
- [fn-pipeline-composition](fn-pipeline-composition.md) - Compose sequential data transformations as an explicit pipeline
- [anti-god-object](anti-god-object.md) - Avoid classes that accumulate unrelated responsibilities
