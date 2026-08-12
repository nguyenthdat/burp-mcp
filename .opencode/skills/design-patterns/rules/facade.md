# facade

> Expose a small, workflow-oriented entry point over a complex subsystem, without hiding the subsystem from users who need it directly.

## Intent & Pressure

Reach for Facade when a subsystem has many interacting parts (auth, retries, serialization, caching, connection setup) and most callers only need one or two common workflows. The pressure is onboarding/readability cost: new callers shouldn't need to learn the whole subsystem to perform the common case.

Do not reach for it when the subsystem is already small — an extra layer just for the sake of "an entry point" adds indirection without payoff. A Facade should not become the *only* way to reach the subsystem; keep the lower-level APIs available for advanced callers who need them.

## Native-Construct Alternative

A well-named module or a couple of top-level functions ("orchestration functions") often provide the same simplification without a formal class. Reach for a Facade type specifically when the simplified workflow needs to hold configuration/state across calls (a session, a connection pool).

## Language Implementations

### Rust

```rust
pub struct VideoConverter {
    codec: CodecEngine,
    muxer: Muxer,
    metadata: MetadataReader,
}

impl VideoConverter {
    pub fn convert(&self, input: &Path, output: &Path) -> Result<(), ConvertError> {
        let meta = self.metadata.read(input)?;
        let decoded = self.codec.decode(input, &meta)?;
        let encoded = self.codec.encode(&decoded, &meta.target_format())?;
        self.muxer.write(output, &encoded)
    }
}
```

The three collaborators (`codec`, `muxer`, `metadata`) remain public/usable directly for advanced callers; `VideoConverter::convert` is the 90%-case entry point.

### TypeScript

```typescript
class VideoConverter {
  constructor(
    private codec: CodecEngine,
    private muxer: Muxer,
    private metadata: MetadataReader,
  ) {}

  async convert(input: string, output: string): Promise<void> {
    const meta = await this.metadata.read(input);
    const decoded = await this.codec.decode(input, meta);
    const encoded = await this.codec.encode(decoded, meta.targetFormat());
    await this.muxer.write(output, encoded);
  }
}
```

### Python

```python
class VideoConverter:
    def __init__(self, codec: CodecEngine, muxer: Muxer, metadata: MetadataReader) -> None:
        self._codec = codec
        self._muxer = muxer
        self._metadata = metadata

    def convert(self, input_path: str, output_path: str) -> None:
        meta = self._metadata.read(input_path)
        decoded = self._codec.decode(input_path, meta)
        encoded = self._codec.encode(decoded, meta.target_format())
        self._muxer.write(output_path, encoded)
```

### Go

```go
type VideoConverter struct {
    codec    CodecEngine
    muxer    Muxer
    metadata MetadataReader
}

func (v VideoConverter) Convert(inputPath, outputPath string) error {
    meta, err := v.metadata.Read(inputPath)
    if err != nil {
        return err
    }
    decoded, err := v.codec.Decode(inputPath, meta)
    if err != nil {
        return err
    }
    encoded, err := v.codec.Encode(decoded, meta.TargetFormat())
    if err != nil {
        return err
    }
    return v.muxer.Write(outputPath, encoded)
}
```

### C#

```csharp
public sealed class VideoConverter
{
    private readonly CodecEngine _codec;
    private readonly Muxer _muxer;
    private readonly MetadataReader _metadata;

    public VideoConverter(CodecEngine codec, Muxer muxer, MetadataReader metadata)
    {
        _codec = codec; _muxer = muxer; _metadata = metadata;
    }

    public async Task ConvertAsync(string input, string output)
    {
        var meta = await _metadata.ReadAsync(input);
        var decoded = await _codec.DecodeAsync(input, meta);
        var encoded = await _codec.EncodeAsync(decoded, meta.TargetFormat());
        await _muxer.WriteAsync(output, encoded);
    }
}
```

### Kotlin

```kotlin
class VideoConverter(
    private val codec: CodecEngine,
    private val muxer: Muxer,
    private val metadata: MetadataReader,
) {
    suspend fun convert(input: String, output: String) {
        val meta = metadata.read(input)
        val decoded = codec.decode(input, meta)
        val encoded = codec.encode(decoded, meta.targetFormat())
        muxer.write(output, encoded)
    }
}
```

### C

```c
typedef struct video_converter {
    codec_engine_t    *codec;
    muxer_t           *muxer;
    metadata_reader_t *metadata;
} video_converter_t;

int video_converter_convert(video_converter_t *vc, const char *input, const char *output) {
    metadata_t meta;
    if (metadata_reader_read(vc->metadata, input, &meta) != 0) return -1;
    decoded_t decoded;
    if (codec_decode(vc->codec, input, &meta, &decoded) != 0) return -1;
    encoded_t encoded;
    if (codec_encode(vc->codec, &decoded, meta.target_format, &encoded) != 0) return -1;
    return muxer_write(vc->muxer, output, &encoded);
}
```

### C++

```cpp
class VideoConverter {
public:
    VideoConverter(CodecEngine &codec, Muxer &muxer, MetadataReader &metadata)
        : codec_(codec), muxer_(muxer), metadata_(metadata) {}

    void convert(const std::filesystem::path &input, const std::filesystem::path &output) {
        auto meta = metadata_.read(input);
        auto decoded = codec_.decode(input, meta);
        auto encoded = codec_.encode(decoded, meta.targetFormat());
        muxer_.write(output, encoded);
    }
private:
    CodecEngine &codec_;
    Muxer &muxer_;
    MetadataReader &metadata_;
};
```

### Swift

```swift
struct VideoConverter {
    let codec: CodecEngine
    let muxer: Muxer
    let metadata: MetadataReader

    func convert(input: URL, output: URL) async throws {
        let meta = try await metadata.read(input)
        let decoded = try await codec.decode(input, meta)
        let encoded = try await codec.encode(decoded, meta.targetFormat())
        try await muxer.write(output, encoded)
    }
}
```

## Pitfalls

- Letting the Facade grow into a god object that absorbs business logic instead of pure orchestration.
- Hiding the subsystem's lower-level APIs entirely, forcing advanced callers to work around the facade instead of dropping to the layer below it.
- A Facade that swallows/generalizes errors from each subsystem step into one vague error, losing the ability to diagnose which step failed.
- Adding a Facade over a subsystem that's already simple, just to have "an entry point."

## See Also

- [adapter](adapter.md) — translating one interface to another, versus simplifying access to several.
- [mediator](mediator.md) — coordinating peer components' interactions, versus presenting a simplified read/write API over them.
- [pipeline-middleware](pipeline-middleware.md) — when the "simplified workflow" is really a composable ordered pipeline.
