# decorator

> Layer behavior around an existing interface, with each layer forwarding to the next, so wrapper order and composition are explicit.

## Intent & Pressure

Reach for Decorator when behavior must be added around an object without subclassing every combination (buffered + compressed + encrypted stream; logged + retried + cached client), and the set of wrappers is open-ended or must be composed in different orders at runtime. The pressure is combinational behavior layering on a shared interface, where the standard library itself often already models this (`BufReader<R>`, `io.Writer` wrapping, HTTP middleware).

Do not reach for it when there's exactly one fixed combination of behavior — just write one concrete implementation. Do not confuse Decorator with Proxy: Decorator's job is to *add* behavior/data, Proxy's job is to *control access* to the same behavior.

## Native-Construct Alternative

A single function that composes the extra behavior inline (`logged(retried(fetch))` as plain function composition) is often clearer than a class hierarchy when there's no need to hold the wrapped object long-term or swap layers at runtime.

## Language Implementations

### Rust

```rust
trait DataSource {
    fn read(&self) -> Result<Vec<u8>, IoError>;
}

struct FileSource { path: PathBuf }
impl DataSource for FileSource {
    fn read(&self) -> Result<Vec<u8>, IoError> { /* ... */ Ok(vec![]) }
}

struct CompressingSource<S: DataSource> { inner: S }
impl<S: DataSource> DataSource for CompressingSource<S> {
    fn read(&self) -> Result<Vec<u8>, IoError> {
        let raw = self.inner.read()?;
        Ok(compress(&raw))
    }
}
```

A generic wrapper `S: DataSource` composes statically with zero overhead, mirroring `BufReader<R>`; use `Box<dyn DataSource>` only when the stack of wrappers is chosen at runtime.

### TypeScript

```typescript
interface DataSource {
  read(): Promise<Buffer>;
}

class CompressingSource implements DataSource {
  constructor(private inner: DataSource) {}
  async read(): Promise<Buffer> {
    const raw = await this.inner.read();
    return compress(raw);
  }
}

const source: DataSource = new CompressingSource(new FileSource(path));
```

### Python

```python
from functools import wraps

def logged(fn):
    @wraps(fn)
    def wrapper(*args, **kwargs):
        result = fn(*args, **kwargs)
        log(f"{fn.__name__} -> {result!r}")
        return result
    return wrapper

@logged
def fetch(url: str) -> bytes: ...
```

Python's `@decorator` syntax is a first-class language feature for exactly this pattern for functions; use a wrapper class implementing the same protocol when the "decorated" thing is an object with multiple methods and state.

### Go

```go
type DataSource interface {
    Read() ([]byte, error)
}

type compressingSource struct{ inner DataSource }

func (c compressingSource) Read() ([]byte, error) {
    raw, err := c.inner.Read()
    if err != nil {
        return nil, err
    }
    return compress(raw), nil
}

func WithCompression(inner DataSource) DataSource {
    return compressingSource{inner: inner}
}
```

Go's `io.Reader`/`io.Writer` wrapping (`bufio.NewReader(r)`, `gzip.NewWriter(w)`) is the standard-library model to mirror for custom decorators.

### C#

```csharp
public interface IDataSource
{
    Task<byte[]> ReadAsync();
}

public sealed class CompressingSource : IDataSource
{
    private readonly IDataSource _inner;
    public CompressingSource(IDataSource inner) => _inner = inner;

    public async Task<byte[]> ReadAsync()
    {
        var raw = await _inner.ReadAsync();
        return Compress(raw);
    }
}
```

### Kotlin

```kotlin
interface DataSource {
    suspend fun read(): ByteArray
}

class CompressingSource(private val inner: DataSource) : DataSource {
    override suspend fun read(): ByteArray = compress(inner.read())
}
```

### C

```c
typedef struct data_source {
    int (*read)(struct data_source *self, uint8_t **out, size_t *out_len);
    void *state;
} data_source_t;

typedef struct { data_source_t *inner; } compressing_state_t;

static int compressing_read(data_source_t *self, uint8_t **out, size_t *out_len) {
    compressing_state_t *state = self->state;
    uint8_t *raw; size_t raw_len;
    int rc = state->inner->read(state->inner, &raw, &raw_len);
    if (rc != 0) return rc;
    return compress(raw, raw_len, out, out_len);
}
```

The wrapper holds a pointer to the inner `data_source_t` and forwards through the same function-pointer contract — the manual-vtable equivalent of an interface wrapper.

### C++

```cpp
class DataSource {
public:
    virtual ~DataSource() = default;
    virtual std::vector<std::byte> read() = 0;
};

class CompressingSource : public DataSource {
public:
    explicit CompressingSource(std::unique_ptr<DataSource> inner) : inner_(std::move(inner)) {}
    std::vector<std::byte> read() override {
        auto raw = inner_->read();
        return compress(raw);
    }
private:
    std::unique_ptr<DataSource> inner_;
};
```

### Swift

```swift
protocol DataSource {
    func read() async throws -> Data
}

struct CompressingSource: DataSource {
    let inner: DataSource
    func read() async throws -> Data {
        let raw = try await inner.read()
        return compress(raw)
    }
}
```

## Pitfalls

- Forwarding only some of the inner interface's methods, silently dropping behavior a wrapped layer was supposed to preserve.
- Losing or double-wrapping errors as they pass through multiple layers.
- Making wrapper order matter silently instead of documenting it (e.g., compress-then-encrypt vs. encrypt-then-compress produce very different results).
- Using Decorator when Proxy is really what's needed — decorators should add capability, not restrict/gate existing capability.
- Deep decorator stacks that make debugging painful; keep a bounded, well-understood number of layers.

## See Also

- [proxy](proxy.md) — same interface wrapping, but controlling access rather than adding behavior.
- [composite](composite.md) — aggregating many objects, versus wrapping one.
- [pipeline-middleware](pipeline-middleware.md) — the request/response-pipeline generalization of Decorator.
