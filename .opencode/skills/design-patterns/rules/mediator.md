# mediator

> Centralize chaotic peer-to-peer coordination between components in one owner, so components reference the mediator instead of each other.

## Intent & Pressure

Reach for Mediator when several components would otherwise need direct references to each other to coordinate (a form with fields that enable/disable one another, a chat room broadcasting to participants, air-traffic-control-style coordination), and that web of pairwise references is growing unmanageable. The pressure is an O(n²) coupling problem: adding component (n+1) shouldn't require updating every existing component's references.

Do not reach for it when there are only two components — a direct call between them is simpler than routing through a mediator. Do not let the mediator become a god object that also contains all the business logic each component should own; its job is coordination, not the components' internal behavior.

## Native-Construct Alternative

For exactly two collaborators, a direct method call or callback is enough. For loosely coupled fire-and-forget coordination across many components, consider [observer](observer.md)/[pub-sub](pub-sub.md) instead — Mediator earns its keep specifically when coordination logic needs a single, stateful, request/response-aware owner.

## Language Implementations

### Rust

```rust
trait Mediator {
    fn notify(&mut self, sender: ComponentId, event: Event);
}

struct FormMediator {
    submit_button: Button,
    checkbox: Checkbox,
}

impl Mediator for FormMediator {
    fn notify(&mut self, sender: ComponentId, event: Event) {
        if sender == ComponentId::Checkbox && event == Event::Toggled {
            self.submit_button.set_enabled(self.checkbox.is_checked());
        }
    }
}
```

Components hold no reference back to the mediator that outlives one call; pass `&mut dyn Mediator` into the operation that needs to notify it, rather than storing it permanently.

### TypeScript

```typescript
interface Mediator {
  notify(sender: ComponentId, event: Event): void;
}

class FormMediator implements Mediator {
  constructor(private submitButton: Button, private checkbox: Checkbox) {}
  notify(sender: ComponentId, event: Event): void {
    if (sender === "checkbox" && event === "toggled") {
      this.submitButton.setEnabled(this.checkbox.isChecked());
    }
  }
}
```

### Python

```python
class Mediator(Protocol):
    def notify(self, sender: str, event: str) -> None: ...

class FormMediator:
    def __init__(self, submit_button: Button, checkbox: Checkbox) -> None:
        self._submit_button = submit_button
        self._checkbox = checkbox

    def notify(self, sender: str, event: str) -> None:
        if sender == "checkbox" and event == "toggled":
            self._submit_button.set_enabled(self._checkbox.is_checked())
```

### Go

```go
type Mediator interface {
    Notify(sender ComponentID, event Event)
}

type formMediator struct {
    submitButton Button
    checkbox     Checkbox
}

func (m *formMediator) Notify(sender ComponentID, event Event) {
    if sender == ComponentCheckbox && event == EventToggled {
        m.submitButton.SetEnabled(m.checkbox.IsChecked())
    }
}
```

### C#

```csharp
public interface IMediator
{
    void Notify(ComponentId sender, Event evt);
}

public sealed class FormMediator : IMediator
{
    private readonly Button _submitButton;
    private readonly Checkbox _checkbox;
    public FormMediator(Button submitButton, Checkbox checkbox)
    {
        _submitButton = submitButton; _checkbox = checkbox;
    }

    public void Notify(ComponentId sender, Event evt)
    {
        if (sender == ComponentId.Checkbox && evt == Event.Toggled)
        {
            _submitButton.SetEnabled(_checkbox.IsChecked);
        }
    }
}
```

### Kotlin

```kotlin
interface Mediator {
    fun notify(sender: ComponentId, event: Event)
}

class FormMediator(
    private val submitButton: Button,
    private val checkbox: Checkbox,
) : Mediator {
    override fun notify(sender: ComponentId, event: Event) {
        if (sender == ComponentId.CHECKBOX && event == Event.TOGGLED) {
            submitButton.setEnabled(checkbox.isChecked)
        }
    }
}
```

### C

```c
typedef struct form_mediator {
    button_t   *submit_button;
    checkbox_t *checkbox;
} form_mediator_t;

void form_mediator_notify(form_mediator_t *m, component_id_t sender, event_t event) {
    if (sender == COMPONENT_CHECKBOX && event == EVENT_TOGGLED) {
        button_set_enabled(m->submit_button, checkbox_is_checked(m->checkbox));
    }
}
```

### C++

```cpp
class Mediator {
public:
    virtual ~Mediator() = default;
    virtual void notify(ComponentId sender, Event event) = 0;
};

class FormMediator : public Mediator {
public:
    FormMediator(Button &submitButton, Checkbox &checkbox)
        : submitButton_(submitButton), checkbox_(checkbox) {}

    void notify(ComponentId sender, Event event) override {
        if (sender == ComponentId::Checkbox && event == Event::Toggled) {
            submitButton_.setEnabled(checkbox_.isChecked());
        }
    }
private:
    Button &submitButton_;
    Checkbox &checkbox_;
};
```

### Swift

```swift
protocol Mediator: AnyObject {
    func notify(sender: ComponentId, event: Event)
}

final class FormMediator: Mediator {
    private let submitButton: Button
    private let checkbox: Checkbox

    init(submitButton: Button, checkbox: Checkbox) {
        self.submitButton = submitButton
        self.checkbox = checkbox
    }

    func notify(sender: ComponentId, event: Event) {
        if sender == .checkbox, event == .toggled {
            submitButton.setEnabled(checkbox.isChecked)
        }
    }
}
```

Components should hold their mediator reference as `weak` to avoid a retain cycle, since the mediator typically owns the components.

## Pitfalls

- Letting the mediator grow into a god object containing every component's business logic instead of pure coordination.
- Cyclic strong references between mediator and components causing memory leaks in reference-counted languages (use `weak`/`Weak` on the back-reference).
- Silent reentrancy: a notification triggers another notification synchronously, causing unexpected recursive coordination.
- No clear ownership of failure handling when one component's reaction to a mediator event throws/errors.

## See Also

- [observer](observer.md) — a looser coordination style: components don't know about a coordinating owner at all.
- [facade](facade.md) — simplifying access to a subsystem, versus centralizing peer coordination within it.
- [command](command.md) — encoding what the mediator dispatches as first-class request objects.
