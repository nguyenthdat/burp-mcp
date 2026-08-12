# template-method

> Fix an algorithm's overall skeleton while letting a few well-defined steps vary in each concrete implementation.

## Intent & Pressure

Reach for Template Method when several implementations share the exact same sequence of steps (validate → fetch → transform → save) and only a handful of those steps genuinely differ between implementations, while the ordering, error handling, and glue logic must stay identical everywhere. The pressure is duplication of the *skeleton* across implementations that each reimplement the whole sequence just to customize one or two steps.

Do not reach for it when the varying step is the *only* thing that differs — a function or [strategy](strategy.md) parameter accepting a callback for that one step is simpler and doesn't require a base class/inheritance-shaped hierarchy at all. Do not use it in languages/teams where composition is strongly preferred over inheritance (Rust, Go) — implement the "template" as a free function taking the varying steps as parameters instead.

## Native-Construct Alternative

A free function that accepts the varying steps as function/closure parameters gives the same fixed-skeleton-with-hooks behavior without any inheritance: `fn run_pipeline(fetch: impl Fn() -> Data, transform: impl Fn(Data) -> Data)`. Prefer this in languages without idiomatic class inheritance, and even in OO languages when there's only one or two hook points.

## Language Implementations

### Rust

```rust
fn run_pipeline(
    fetch: impl Fn() -> Result<Data, PipelineError>,
    transform: impl Fn(Data) -> Data,
    save: impl Fn(Data) -> Result<(), PipelineError>,
) -> Result<(), PipelineError> {
    let data = fetch()?;
    let transformed = transform(data);
    save(transformed)
}
```

Rust has no class inheritance, so the "template" is a free function taking the varying steps as closures — composition, not a base-trait-with-default-methods hierarchy.

### TypeScript

```typescript
abstract class ImportPipeline {
  run(): void {
    const data = this.fetch();
    const transformed = this.transform(data);
    this.save(transformed);
  }
  protected abstract fetch(): Data;
  protected abstract transform(data: Data): Data;
  protected abstract save(data: Data): void;
}

class CsvImportPipeline extends ImportPipeline {
  protected fetch(): Data { /* ... */ return {} as Data; }
  protected transform(data: Data): Data { return data; }
  protected save(data: Data): void { /* ... */ }
}
```

### Python

```python
from abc import ABC, abstractmethod

class ImportPipeline(ABC):
    def run(self) -> None:
        data = self.fetch()
        transformed = self.transform(data)
        self.save(transformed)

    @abstractmethod
    def fetch(self) -> Data: ...
    @abstractmethod
    def transform(self, data: Data) -> Data: ...
    @abstractmethod
    def save(self, data: Data) -> None: ...
```

### Go

```go
type Steps struct {
    Fetch     func() (Data, error)
    Transform func(Data) Data
    Save      func(Data) error
}

func RunPipeline(steps Steps) error {
    data, err := steps.Fetch()
    if err != nil {
        return err
    }
    transformed := steps.Transform(data)
    return steps.Save(transformed)
}
```

Go has no inheritance either; a struct of function fields (or plain parameters) is the idiomatic template.

### C#

```csharp
public abstract class ImportPipeline
{
    public void Run()
    {
        var data = Fetch();
        var transformed = Transform(data);
        Save(transformed);
    }
    protected abstract Data Fetch();
    protected abstract Data Transform(Data data);
    protected abstract void Save(Data data);
}
```

### Kotlin

```kotlin
abstract class ImportPipeline {
    fun run() {
        val data = fetch()
        val transformed = transform(data)
        save(transformed)
    }
    protected abstract fun fetch(): Data
    protected abstract fun transform(data: Data): Data
    protected abstract fun save(data: Data)
}
```

### C

```c
typedef struct pipeline_steps {
    data_t (*fetch)(void);
    data_t (*transform)(data_t data);
    int (*save)(data_t data);
} pipeline_steps_t;

int run_pipeline(const pipeline_steps_t *steps) {
    data_t data = steps->fetch();
    data_t transformed = steps->transform(data);
    return steps->save(transformed);
}
```

### C++

```cpp
class ImportPipeline {
public:
    virtual ~ImportPipeline() = default;
    void run() {
        auto data = fetch();
        auto transformed = transform(data);
        save(transformed);
    }
protected:
    virtual Data fetch() = 0;
    virtual Data transform(Data data) = 0;
    virtual void save(Data data) = 0;
};
```

### Swift

```swift
protocol ImportPipeline {
    func fetch() -> Data
    func transform(_ data: Data) -> Data
    func save(_ data: Data)
}

extension ImportPipeline {
    func run() {
        let data = fetch()
        let transformed = transform(data)
        save(transformed)
    }
}
```

A protocol extension supplying the default `run()` implementation gives Swift a Template Method without a class hierarchy — any conforming `struct` or `class` gets the fixed skeleton for free.

## Pitfalls

- Using inheritance-based Template Method when a single-hook case would be simpler as a plain function with a callback parameter.
- Hook methods with unclear call order or unclear which are required versus optional — document both explicitly.
- A "hook" that silently changes the meaning of the skeleton (e.g., skipping `save` under some condition) without that being visible from the base algorithm.
- Deep template-method hierarchies (subclass of a subclass of a subclass) that make the actual execution order hard to trace.
- Sharing mutable state between steps via base-class fields when passing it explicitly through parameters/return values would be clearer and testable in isolation.

## See Also

- [strategy](strategy.md) — swapping a single step or the whole algorithm via injection, rather than fixing a skeleton with hook methods.
- [factory-method](factory-method.md) — often one of the "hook" steps in a Template Method (deferring how one step's object is created).
- [pipeline-middleware](pipeline-middleware.md) — a more dynamic, composable version of a fixed multi-step pipeline.
