# prototype

> Produce a new object by copying an existing one instead of rebuilding it from scratch.

## Intent & Pressure

Reach for Prototype when constructing a value from raw inputs is expensive, complex, or requires knowledge the caller doesn't have (a parsed configuration, a populated cache entry, a scene-graph node with many nested children), but a copy of an existing instance is cheap and sufficient. It's also useful when a factory has to return objects it received only as an abstract interface, and copying is the only way to obtain a new independent instance without a concrete constructor.

Do not reach for it when the built-in copy/clone facility already does exactly this — which is most of the time. The design decision that actually matters is not "add a `clone()` method" (languages already have one) but **whether cloning is deep or shallow**, and documenting that clearly.

## Native-Construct Alternative

Use the language's own copy facility and just be explicit about depth: `#[derive(Clone)]` (Rust), `copy.deepcopy`/`dataclasses.replace` (Python), a copy constructor or `with` expression (C#/records), `structuredClone`/spread (TypeScript), `.copy()` on Kotlin `data class`, `Value` semantics on Swift `struct`/`enum` (already prototype-like by default). Reach for a dedicated `Prototype` interface only when the concrete type is unknown to the caller and cloning must happen through an abstraction.

## Language Implementations

### Rust

```rust
#[derive(Clone)]
struct DocumentTemplate {
    title: String,
    sections: Vec<Section>, // deep clone: each Section is cloned independently
}

let base = DocumentTemplate { title: "Invoice".into(), sections: vec![] };
let draft = base.clone(); // independent copy; mutate `draft` freely
```

`Clone` is Rust's Prototype. Document explicitly when a field is `Arc<T>`/`Rc<T>` — cloning shares that data rather than duplicating it.

### TypeScript

```typescript
interface Prototype<T> {
  clone(): T;
}

class DocumentTemplate implements Prototype<DocumentTemplate> {
  constructor(public title: string, public sections: Section[]) {}

  clone(): DocumentTemplate {
    return new DocumentTemplate(
      this.title,
      this.sections.map((s) => s.clone()), // deep copy nested sections
    );
  }
}
```

### Python

```python
import copy
from dataclasses import dataclass, replace

@dataclass
class DocumentTemplate:
    title: str
    sections: list["Section"]

    def clone(self) -> "DocumentTemplate":
        return copy.deepcopy(self)

# Shallow variant for immutable-field tweaks:
draft = replace(base, title="Invoice Draft")
```

### Go

```go
type DocumentTemplate struct {
    Title    string
    Sections []Section
}

func (d DocumentTemplate) Clone() DocumentTemplate {
    sections := make([]Section, len(d.Sections))
    copy(sections, d.Sections) // shallow; deep-copy each Section if it holds pointers/slices
    return DocumentTemplate{Title: d.Title, Sections: sections}
}
```

Go has no built-in deep copy; be explicit about which fields need element-wise copying versus a plain slice/struct copy.

### C#

```csharp
public sealed record DocumentTemplate(string Title, IReadOnlyList<Section> Sections)
{
    public DocumentTemplate Clone() =>
        this with { Sections = Sections.Select(s => s.Clone()).ToList() };
}

// shallow copy via `with` for simple field tweaks:
var draft = base with { Title = "Invoice Draft" };
```

### Kotlin

```kotlin
data class DocumentTemplate(val title: String, val sections: List<Section>) {
    fun deepClone(): DocumentTemplate = copy(sections = sections.map { it.deepClone() })
}

// shallow copy is built in:
val draft = base.copy(title = "Invoice Draft")
```

### C

```c
typedef struct document_template {
    char *title;
    section_t *sections;
    size_t section_count;
} document_template_t;

document_template_t *document_clone(const document_template_t *src) {
    document_template_t *copy = malloc(sizeof(*copy));
    if (!copy) return NULL;
    copy->title = strdup(src->title);
    copy->section_count = src->section_count;
    copy->sections = malloc(sizeof(section_t) * src->section_count);
    for (size_t i = 0; i < src->section_count; i++) {
        copy->sections[i] = section_clone(&src->sections[i]); /* deep */
    }
    return copy;
}
/* caller must call document_destroy(copy) */
```

C has to hand-write every level of a deep copy; document exactly which pointers are duplicated versus shared.

### C++

```cpp
class Prototype {
public:
    virtual ~Prototype() = default;
    virtual std::unique_ptr<Prototype> clone() const = 0;
};

class DocumentTemplate : public Prototype {
public:
    std::unique_ptr<Prototype> clone() const override {
        return std::make_unique<DocumentTemplate>(*this); // copy ctor: deep-copies value members
    }
private:
    std::string title_;
    std::vector<Section> sections_;
};
```

A virtual `clone()` is the classic C++ Prototype, needed specifically when code holds a `Prototype*`/`unique_ptr<Prototype>` and doesn't know the concrete type. Otherwise the copy constructor alone suffices.

### Swift

```swift
struct DocumentTemplate {
    var title: String
    var sections: [Section]
}

let base = DocumentTemplate(title: "Invoice", sections: [])
var draft = base // value semantics: already an independent copy, no explicit clone needed
draft.title = "Invoice Draft"
```

Swift `struct`/`enum` value types are Prototype "for free" via copy-on-write. Reserve an explicit `clone()` for reference types (`class`) that need deep copying.

## Pitfalls

- Assuming a shallow copy/clone gives independence when a field is a shared handle (`Arc`, `Rc`, JS object reference, Python mutable default) — that shares state instead of duplicating it.
- Deep-cloning recursively without a depth or cycle guard on self-referential structures.
- Forgetting resource duplication has a real cost (file handles, sockets, DB connections) that shouldn't be cloned at all — exclude or re-acquire those explicitly.
- Using Prototype where a value type with structural equality (Swift/Kotlin/records) already provides copy-with-mutation for free.

## See Also

- [builder](builder.md) — when the copy needs several fields changed together with validation.
- [flyweight](flyweight.md) — the opposite pressure: sharing rather than duplicating large immutable state.
- [memento](memento.md) — capturing state for restoration rather than for producing a new working instance.
