# composite

> Treat individual objects and compositions of objects through the same interface, so clients don't need to distinguish leaf from branch.

## Intent & Pressure

Reach for Composite when you have a recursive part-whole hierarchy (file system entries, UI widget trees, org charts, arithmetic expression trees) and clients need to run the same operation (render, total size, validate) over a single leaf or an entire subtree without special-casing. The pressure is uniform recursive traversal over a genuinely tree-shaped domain.

Do not reach for it when the data is really just a flat list with a grouping key — that doesn't need recursive composition. Do not build a generic Composite for a hierarchy that is always exactly two levels deep; a simple parent/children pair is enough.

## Native-Construct Alternative

For a closed, small set of node kinds, a recursive `enum`/sealed-class hierarchy with exhaustive matching is usually clearer and safer than a `Composite` base class, because the compiler enforces the node set and every recursive function must handle every kind.

## Language Implementations

### Rust

```rust
enum Node {
    File { name: String, size: u64 },
    Directory { name: String, children: Vec<Node> },
}

impl Node {
    fn total_size(&self) -> u64 {
        match self {
            Node::File { size, .. } => *size,
            Node::Directory { children, .. } => children.iter().map(Node::total_size).sum(),
        }
    }
}
```

A recursive `enum` is the idiomatic Composite for a closed node set; use `Vec<Box<dyn Component>>` only when node kinds are open/plugin-defined.

### TypeScript

```typescript
interface Component {
  totalSize(): number;
}

class FileNode implements Component {
  constructor(private size: number) {}
  totalSize(): number { return this.size; }
}

class DirectoryNode implements Component {
  private children: Component[] = [];
  add(child: Component): void { this.children.push(child); }
  totalSize(): number {
    return this.children.reduce((sum, c) => sum + c.totalSize(), 0);
  }
}
```

### Python

```python
from abc import ABC, abstractmethod

class Component(ABC):
    @abstractmethod
    def total_size(self) -> int: ...

class FileNode(Component):
    def __init__(self, size: int) -> None:
        self.size = size
    def total_size(self) -> int:
        return self.size

class DirectoryNode(Component):
    def __init__(self) -> None:
        self.children: list[Component] = []
    def total_size(self) -> int:
        return sum(child.total_size() for child in self.children)
```

### Go

```go
type Component interface {
    TotalSize() int64
}

type FileNode struct{ Size int64 }
func (f FileNode) TotalSize() int64 { return f.Size }

type DirectoryNode struct{ Children []Component }
func (d DirectoryNode) TotalSize() int64 {
    var total int64
    for _, c := range d.Children {
        total += c.TotalSize()
    }
    return total
}
```

### C#

```csharp
public interface IComponent
{
    long TotalSize();
}

public sealed class FileNode : IComponent
{
    private readonly long _size;
    public FileNode(long size) => _size = size;
    public long TotalSize() => _size;
}

public sealed class DirectoryNode : IComponent
{
    private readonly List<IComponent> _children = new();
    public void Add(IComponent child) => _children.Add(child);
    public long TotalSize() => _children.Sum(c => c.TotalSize());
}
```

### Kotlin

```kotlin
sealed interface Component {
    fun totalSize(): Long
}

data class FileNode(val size: Long) : Component {
    override fun totalSize() = size
}

data class DirectoryNode(val children: List<Component>) : Component {
    override fun totalSize() = children.sumOf { it.totalSize() }
}
```

A `sealed interface` plus `when` gives exhaustiveness checking, the Kotlin equivalent of Rust's `enum`+`match` recursive Composite.

### C

```c
typedef enum { NODE_FILE, NODE_DIRECTORY } node_kind_t;

typedef struct node {
    node_kind_t kind;
    union {
        struct { uint64_t size; } file;
        struct { struct node **children; size_t count; } directory;
    } as;
} node_t;

uint64_t node_total_size(const node_t *n) {
    switch (n->kind) {
        case NODE_FILE:
            return n->as.file.size;
        case NODE_DIRECTORY: {
            uint64_t total = 0;
            for (size_t i = 0; i < n->as.directory.count; i++) {
                total += node_total_size(n->as.directory.children[i]);
            }
            return total;
        }
    }
    return 0;
}
```

A tagged union with a `switch` is C's Composite for a closed node set; guard recursion depth explicitly since C has no automatic stack-overflow recovery.

### C++

```cpp
class Component {
public:
    virtual ~Component() = default;
    virtual std::uint64_t totalSize() const = 0;
};

class FileNode : public Component {
public:
    explicit FileNode(std::uint64_t size) : size_(size) {}
    std::uint64_t totalSize() const override { return size_; }
private:
    std::uint64_t size_;
};

class DirectoryNode : public Component {
public:
    void add(std::unique_ptr<Component> child) { children_.push_back(std::move(child)); }
    std::uint64_t totalSize() const override {
        std::uint64_t total = 0;
        for (const auto &child : children_) total += child->totalSize();
        return total;
    }
private:
    std::vector<std::unique_ptr<Component>> children_;
};
```

### Swift

```swift
indirect enum Node {
    case file(size: Int)
    case directory(children: [Node])

    var totalSize: Int {
        switch self {
        case .file(let size): return size
        case .directory(let children): return children.reduce(0) { $0 + $1.totalSize }
        }
    }
}
```

`indirect enum` gives Swift a value-type recursive Composite with exhaustive `switch`, avoiding reference-type node allocation entirely for a closed node set.

## Pitfalls

- Letting a leaf silently accept child-management calls (`add`/`remove`) that make no sense for it — either split the interface or make those calls no-ops with a clear contract, not a crash.
- Unbounded recursion without a depth or cycle check on user-supplied trees.
- Choosing an open `Vec<Box<dyn Component>>`-style Composite for a node set that is actually closed and small, giving up exhaustiveness checking for no benefit.
- Mutating shared child references concurrently without synchronization when the tree is accessed from multiple threads.

## See Also

- [decorator](decorator.md) — both wrap a common interface, but Decorator layers behavior around one object rather than aggregating many.
- [visitor](visitor.md) — adding new operations over a Composite's node types without modifying each node class.
- [iterator](iterator.md) — traversing a Composite's elements without exposing its recursive structure.
