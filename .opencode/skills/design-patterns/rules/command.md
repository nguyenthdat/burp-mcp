# command

> Turn a request into a first-class object or value that can be queued, logged, retried, or undone.

## Intent & Pressure

Reach for Command when a request needs to outlive the call that created it — queued for later execution, logged for audit, sent across a process boundary, retried, or undone. The pressure is that "just calling the function" isn't enough because the request itself needs to be stored, inspected, or reversed.

Do not reach for it when you can simply call the function immediately and there's no need to store, queue, replay, or undo the request. Do not claim undo support unless the side effects are actually compensable — a `DeleteFileCommand` can't always be undone.

## Native-Construct Alternative

A closure/function value captured with its arguments (`Fn`/`Runnable`/lambda) is sufficient when there's exactly one kind of deferred action and no need for a heterogeneous history or serialization. Promote to an explicit Command enum/interface once you need to store many different kinds of requests uniformly, serialize them, or support undo.

## Language Implementations

### Rust

```rust
enum Command {
    CreateFile { path: PathBuf, contents: Vec<u8> },
    DeleteFile { path: PathBuf },
}

trait Execute {
    fn execute(&self, fs: &mut dyn FileSystem) -> Result<(), CommandError>;
    fn undo(&self, fs: &mut dyn FileSystem) -> Result<(), CommandError>;
}

impl Execute for Command {
    fn execute(&self, fs: &mut dyn FileSystem) -> Result<(), CommandError> {
        match self {
            Command::CreateFile { path, contents } => fs.write(path, contents),
            Command::DeleteFile { path } => fs.remove(path),
        }
    }
    fn undo(&self, fs: &mut dyn FileSystem) -> Result<(), CommandError> {
        match self {
            Command::CreateFile { path, .. } => fs.remove(path),
            Command::DeleteFile { path } => Err(CommandError::CannotUndo(path.clone())),
        }
    }
}
```

A closed `enum` for a fixed command set gives exhaustiveness checking; use a `dyn Execute` trait object only for an open, plugin-defined command set.

### TypeScript

```typescript
interface Command {
  execute(fs: FileSystem): Promise<void>;
  undo(fs: FileSystem): Promise<void>;
}

class CreateFileCommand implements Command {
  constructor(private path: string, private contents: Buffer) {}
  async execute(fs: FileSystem): Promise<void> { await fs.write(this.path, this.contents); }
  async undo(fs: FileSystem): Promise<void> { await fs.remove(this.path); }
}

class CommandHistory {
  private history: Command[] = [];
  async run(cmd: Command, fs: FileSystem): Promise<void> {
    await cmd.execute(fs);
    this.history.push(cmd);
  }
  async undoLast(fs: FileSystem): Promise<void> {
    const cmd = this.history.pop();
    if (cmd) await cmd.undo(fs);
  }
}
```

### Python

```python
from dataclasses import dataclass
from typing import Protocol

class Command(Protocol):
    def execute(self, fs: FileSystem) -> None: ...
    def undo(self, fs: FileSystem) -> None: ...

@dataclass
class CreateFileCommand:
    path: str
    contents: bytes

    def execute(self, fs: FileSystem) -> None:
        fs.write(self.path, self.contents)

    def undo(self, fs: FileSystem) -> None:
        fs.remove(self.path)
```

### Go

```go
type Command interface {
    Execute(fs FileSystem) error
    Undo(fs FileSystem) error
}

type CreateFileCommand struct {
    Path     string
    Contents []byte
}

func (c CreateFileCommand) Execute(fs FileSystem) error { return fs.Write(c.Path, c.Contents) }
func (c CreateFileCommand) Undo(fs FileSystem) error    { return fs.Remove(c.Path) }
```

### C#

```csharp
public interface ICommand
{
    Task ExecuteAsync(IFileSystem fs);
    Task UndoAsync(IFileSystem fs);
}

public sealed class CreateFileCommand : ICommand
{
    private readonly string _path;
    private readonly byte[] _contents;
    public CreateFileCommand(string path, byte[] contents) { _path = path; _contents = contents; }

    public Task ExecuteAsync(IFileSystem fs) => fs.WriteAsync(_path, _contents);
    public Task UndoAsync(IFileSystem fs) => fs.RemoveAsync(_path);
}
```

### Kotlin

```kotlin
interface Command {
    suspend fun execute(fs: FileSystem)
    suspend fun undo(fs: FileSystem)
}

data class CreateFileCommand(val path: String, val contents: ByteArray) : Command {
    override suspend fun execute(fs: FileSystem) = fs.write(path, contents)
    override suspend fun undo(fs: FileSystem) = fs.remove(path)
}
```

### C

```c
typedef struct command {
    int (*execute)(struct command *self, file_system_t *fs);
    int (*undo)(struct command *self, file_system_t *fs);
    void *state;
} command_t;

typedef struct { char *path; uint8_t *contents; size_t len; } create_file_state_t;

int create_file_execute(command_t *self, file_system_t *fs) {
    create_file_state_t *s = self->state;
    return fs_write(fs, s->path, s->contents, s->len);
}
int create_file_undo(command_t *self, file_system_t *fs) {
    create_file_state_t *s = self->state;
    return fs_remove(fs, s->path);
}
```

### C++

```cpp
class Command {
public:
    virtual ~Command() = default;
    virtual void execute(FileSystem &fs) = 0;
    virtual void undo(FileSystem &fs) = 0;
};

class CreateFileCommand : public Command {
public:
    CreateFileCommand(std::filesystem::path path, std::vector<std::byte> contents)
        : path_(std::move(path)), contents_(std::move(contents)) {}
    void execute(FileSystem &fs) override { fs.write(path_, contents_); }
    void undo(FileSystem &fs) override { fs.remove(path_); }
private:
    std::filesystem::path path_;
    std::vector<std::byte> contents_;
};
```

### Swift

```swift
protocol Command {
    func execute(_ fs: FileSystem) async throws
    func undo(_ fs: FileSystem) async throws
}

struct CreateFileCommand: Command {
    let path: String
    let contents: Data
    func execute(_ fs: FileSystem) async throws { try await fs.write(path, contents) }
    func undo(_ fs: FileSystem) async throws { try await fs.remove(path) }
}
```

## Pitfalls

- Claiming undo support for commands whose side effects aren't actually reversible (external emails sent, payments charged).
- Storing a mutable reference to shared context inside the command instead of passing context into `execute`, causing stale-state bugs when the command runs later.
- Re-executing a non-idempotent command on retry without a deduplication/idempotency key.
- Growing command history unbounded with no eviction/checkpoint strategy for long-running sessions.
- Serializing commands for persistence/replay without versioning the schema, breaking replay after a code change.

## See Also

- [memento](memento.md) — capturing state snapshots for restoration, often paired with Command for undo.
- [chain-of-responsibility](chain-of-responsibility.md) — dispatching a command/request through ordered handlers.
- [event-sourcing](event-sourcing.md) — persisting a sequence of commands/events as the system of record.
