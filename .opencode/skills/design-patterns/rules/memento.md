# memento

> Capture and later restore an object's internal state, without exposing that internal state to whoever holds the snapshot.

## Intent & Pressure

Reach for Memento when you need undo/redo, checkpoints, or crash recovery, and the object whose state you're capturing has internals that shouldn't be exposed to the code managing the history (a text editor's undo stack shouldn't need to know the document's internal rope/tree structure). The pressure is state capture with an encapsulation boundary: the "caretaker" holding snapshots can store and restore them, but not inspect or mutate their insides.

Do not reach for it when a full clone of already-public state is sufficient — that's just [prototype](prototype.md). Memento earns its keep specifically when the originator's internal representation must stay hidden from the code that manages the snapshot history.

## Native-Construct Alternative

If the object's state is already fully public and cheap to copy, a plain clone/copy stored in a list is a full Memento without needing a separate opaque snapshot type. Reach for a dedicated (often private) snapshot type when internal fields must stay hidden from the caretaker.

## Language Implementations

### Rust

```rust
struct Document { content: String, cursor: usize }

struct DocumentMemento { content: String, cursor: usize } // private to this module

impl Document {
    fn save(&self) -> DocumentMemento {
        DocumentMemento { content: self.content.clone(), cursor: self.cursor }
    }
    fn restore(&mut self, memento: DocumentMemento) {
        self.content = memento.content;
        self.cursor = memento.cursor;
    }
}

struct History { snapshots: Vec<DocumentMemento> }
impl History {
    fn push(&mut self, m: DocumentMemento) { self.snapshots.push(m); }
    fn pop(&mut self) -> Option<DocumentMemento> { self.snapshots.pop() }
}
```

`DocumentMemento`'s fields stay private to the module; `History` can store and restore mementos but never read their contents.

### TypeScript

```typescript
class DocumentMemento {
  private constructor(private readonly content: string, private readonly cursor: number) {}
  static capture(doc: Document): DocumentMemento {
    return new DocumentMemento(doc.content, doc.cursor);
  }
  restoreInto(doc: Document): void {
    doc.content = this.content;
    doc.cursor = this.cursor;
  }
}

class History {
  private snapshots: DocumentMemento[] = [];
  push(m: DocumentMemento): void { this.snapshots.push(m); }
  pop(): DocumentMemento | undefined { return this.snapshots.pop(); }
}
```

### Python

```python
from dataclasses import dataclass, replace

@dataclass(frozen=True)
class DocumentMemento:
    content: str
    cursor: int

class Document:
    def __init__(self, content: str, cursor: int) -> None:
        self.content, self.cursor = content, cursor

    def save(self) -> DocumentMemento:
        return DocumentMemento(self.content, self.cursor)

    def restore(self, memento: DocumentMemento) -> None:
        self.content, self.cursor = memento.content, memento.cursor
```

### Go

```go
type documentMemento struct { // unexported: only this package can read the fields
    content string
    cursor  int
}

func (d *Document) Save() documentMemento {
    return documentMemento{content: d.content, cursor: d.cursor}
}

func (d *Document) Restore(m documentMemento) {
    d.content, d.cursor = m.content, m.cursor
}
```

Go's package-level unexported fields give the same encapsulation as a private nested type: callers outside the package can hold a `documentMemento` but not read its fields.

### C#

```csharp
public sealed class DocumentMemento
{
    internal string Content { get; }
    internal int Cursor { get; }
    internal DocumentMemento(string content, int cursor) { Content = content; Cursor = cursor; }
}

public sealed class Document
{
    public string Content { get; private set; } = "";
    public int Cursor { get; private set; }

    public DocumentMemento Save() => new(Content, Cursor);
    public void Restore(DocumentMemento memento)
    {
        Content = memento.Content;
        Cursor = memento.Cursor;
    }
}
```

### Kotlin

```kotlin
class DocumentMemento internal constructor(internal val content: String, internal val cursor: Int)

class Document(var content: String, var cursor: Int) {
    fun save(): DocumentMemento = DocumentMemento(content, cursor)
    fun restore(memento: DocumentMemento) {
        content = memento.content
        cursor = memento.cursor
    }
}
```

### C

```c
typedef struct document_memento { /* opaque to callers outside document.c */
    char *content;
    size_t cursor;
} document_memento_t;

document_memento_t *document_save(const document_t *doc) {
    document_memento_t *m = malloc(sizeof(*m));
    m->content = strdup(doc->content);
    m->cursor = doc->cursor;
    return m;
}

void document_restore(document_t *doc, const document_memento_t *m) {
    free(doc->content);
    doc->content = strdup(m->content);
    doc->cursor = m->cursor;
}
/* caller owns the memento and must call document_memento_destroy() */
```

Declare `document_memento_t` fully only in the `.c` file and expose an opaque forward-declared pointer type in the header for true encapsulation.

### C++

```cpp
class Document {
public:
    class Memento {
        friend class Document;
        Memento(std::string content, std::size_t cursor) : content_(std::move(content)), cursor_(cursor) {}
        std::string content_;
        std::size_t cursor_;
    };

    Memento save() const { return Memento(content_, cursor_); }
    void restore(const Memento &m) { content_ = m.content_; cursor_ = m.cursor_; }

private:
    std::string content_;
    std::size_t cursor_ = 0;
};
```

`friend class Document` keeps `Memento`'s constructor and fields inaccessible to any other caller, even though the type itself is public.

### Swift

```swift
struct DocumentMemento {
    fileprivate let content: String
    fileprivate let cursor: Int
}

final class Document {
    private(set) var content: String
    private(set) var cursor: Int

    init(content: String, cursor: Int) { self.content = content; self.cursor = cursor }

    func save() -> DocumentMemento { DocumentMemento(content: content, cursor: cursor) }
    func restore(_ memento: DocumentMemento) {
        content = memento.content
        cursor = memento.cursor
    }
}
```

## Pitfalls

- Exposing the memento's internal fields publicly, defeating the encapsulation the pattern exists to provide.
- Deserializing persisted mementos from an untrusted source without validating the schema and version.
- Unbounded snapshot history with no size/age eviction, leaking memory in long-running sessions.
- Deep-copying large state on every snapshot without considering structural sharing/copy-on-write, causing quadratic memory growth for frequent snapshots.

## See Also

- [command](command.md) — often paired: a command's undo restores a memento captured before execution.
- [prototype](prototype.md) — simple duplication of already-public state, versus encapsulated snapshotting.
- [event-sourcing](event-sourcing.md) — an alternative persistence model that reconstructs state from events instead of storing snapshots directly.
