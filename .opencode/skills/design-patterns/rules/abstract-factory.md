# abstract-factory

> Create families of related products without the caller naming any concrete family.

## Intent & Pressure

Reach for Abstract Factory when a client must construct several related objects that must stay mutually compatible (a UI theme's button + checkbox + scrollbar, a database driver's connection + statement + transaction), and the concrete family is selected once, far from where the objects are used. The pressure is cross-product consistency: picking "dark theme" must not let a "light theme" checkbox slip in.

Do not reach for it when there is only one product in the family, or when the products don't need to be compatible with each other — a single Factory Method per product is simpler. Do not build an Abstract Factory just because there are multiple `create*` functions; the defining pressure is a *family* that must vary together.

## Native-Construct Alternative

A single struct/object/module holding one factory function per product, selected once at startup, often satisfies this without a formal interface hierarchy. Only promote to a full Abstract Factory trait/interface when multiple interchangeable families must implement the same contract and be swapped as a unit (e.g., in tests).

## Language Implementations

### Rust

```rust
trait UiFactory {
    type Button: Button;
    type Checkbox: Checkbox;
    fn create_button(&self) -> Self::Button;
    fn create_checkbox(&self) -> Self::Checkbox;
}

struct DarkFactory;
impl UiFactory for DarkFactory {
    type Button = DarkButton;
    type Checkbox = DarkCheckbox;
    fn create_button(&self) -> DarkButton { DarkButton }
    fn create_checkbox(&self) -> DarkCheckbox { DarkCheckbox }
}
```

Use associated types for a static-dispatch family (zero-cost, one family per generic instantiation). Use `Box<dyn Button>`/`Box<dyn Checkbox>` return types only when the family itself must be chosen at runtime and stored.

### TypeScript

```typescript
interface UiFactory {
  createButton(): Button;
  createCheckbox(): Checkbox;
}

class DarkFactory implements UiFactory {
  createButton(): Button { return new DarkButton(); }
  createCheckbox(): Checkbox { return new DarkCheckbox(); }
}
class LightFactory implements UiFactory {
  createButton(): Button { return new LightButton(); }
  createCheckbox(): Checkbox { return new LightCheckbox(); }
}
```

### Python

```python
from abc import ABC, abstractmethod

class UiFactory(ABC):
    @abstractmethod
    def create_button(self) -> "Button": ...
    @abstractmethod
    def create_checkbox(self) -> "Checkbox": ...

class DarkFactory(UiFactory):
    def create_button(self) -> "Button":
        return DarkButton()
    def create_checkbox(self) -> "Checkbox":
        return DarkCheckbox()
```

### Go

```go
type UIFactory interface {
    CreateButton() Button
    CreateCheckbox() Checkbox
}

type darkFactory struct{}
func (darkFactory) CreateButton() Button     { return DarkButton{} }
func (darkFactory) CreateCheckbox() Checkbox { return DarkCheckbox{} }

type lightFactory struct{}
func (lightFactory) CreateButton() Button     { return LightButton{} }
func (lightFactory) CreateCheckbox() Checkbox { return LightCheckbox{} }
```

### C#

```csharp
public interface IUiFactory
{
    IButton CreateButton();
    ICheckbox CreateCheckbox();
}

public sealed class DarkFactory : IUiFactory
{
    public IButton CreateButton() => new DarkButton();
    public ICheckbox CreateCheckbox() => new DarkCheckbox();
}
```

### Kotlin

```kotlin
interface UiFactory {
    fun createButton(): Button
    fun createCheckbox(): Checkbox
}

class DarkFactory : UiFactory {
    override fun createButton(): Button = DarkButton()
    override fun createCheckbox(): Checkbox = DarkCheckbox()
}
```

### C

```c
typedef struct ui_factory {
    button_t   *(*create_button)(void);
    checkbox_t *(*create_checkbox)(void);
} ui_factory_t;

button_t   *dark_create_button(void)   { /* ... */ }
checkbox_t *dark_create_checkbox(void) { /* ... */ }

static const ui_factory_t dark_factory = {
    .create_button = dark_create_button,
    .create_checkbox = dark_create_checkbox,
};
```

A `static const` struct of function pointers is C's family: swap the whole struct to swap the family, guaranteeing consistency at the call site.

### C++

```cpp
class UiFactory {
public:
    virtual ~UiFactory() = default;
    virtual std::unique_ptr<Button> createButton() const = 0;
    virtual std::unique_ptr<Checkbox> createCheckbox() const = 0;
};

class DarkFactory : public UiFactory {
public:
    std::unique_ptr<Button> createButton() const override { return std::make_unique<DarkButton>(); }
    std::unique_ptr<Checkbox> createCheckbox() const override { return std::make_unique<DarkCheckbox>(); }
};
```

### Swift

```swift
protocol UIFactory {
    func makeButton() -> Button
    func makeCheckbox() -> Checkbox
}

struct DarkFactory: UIFactory {
    func makeButton() -> Button { DarkButton() }
    func makeCheckbox() -> Checkbox { DarkCheckbox() }
}
```

## Pitfalls

- Building an Abstract Factory for products that don't actually need to stay compatible — that's just several unrelated Factory Methods.
- Widening the factory interface every time a new product type is added, breaking every existing family implementation (prefer smaller, focused factories, or default methods where the language supports them).
- Letting call sites downcast a product back to a concrete family type, defeating the abstraction.
- Choosing the family in many scattered places instead of once at the composition root, which reintroduces the mismatch risk the pattern exists to prevent.

## See Also

- [factory-method](factory-method.md) — the single-product building block this pattern composes.
- [builder](builder.md) — when a product itself needs multi-step construction.
- [dependency-injection](dependency-injection.md) — typically how the chosen factory instance reaches consumers.
