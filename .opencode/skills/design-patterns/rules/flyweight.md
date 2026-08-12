# flyweight

> Share large immutable intrinsic state across many logical instances instead of duplicating it, keeping per-instance (extrinsic) state separate.

## Intent & Pressure

Reach for Flyweight when profiling shows real memory pressure from many objects that repeat the same large immutable data (glyph bitmaps in a text renderer, tile textures in a game map, interned strings, parsed-and-cached configuration blocks). The pressure is measured memory cost from duplication of data that is genuinely immutable and shareable.

Do not reach for it without measurement — it trades memory for a more complex ownership model (shared references, interning, cache invalidation) and is easy to introduce prematurely. Do not use it for state that's actually per-instance (extrinsic); mixing that into the shared object reintroduces the bug class it's meant to prevent (aliased mutation).

## Native-Construct Alternative

Just duplicate the data until a profiler identifies real pressure. When it does, prefer language-level string/const interning and a simple keyed cache (`HashMap<Key, Arc<T>>`) over inventing a bespoke Flyweight class hierarchy.

## Language Implementations

### Rust

```rust
struct Glyph {
    bitmap: Arc<[u8]>, // shared intrinsic state
}

struct GlyphCache {
    glyphs: Mutex<HashMap<char, Arc<[u8]>>>,
}

impl GlyphCache {
    fn get(&self, c: char) -> Arc<[u8]> {
        let mut glyphs = self.glyphs.lock().unwrap();
        glyphs.entry(c).or_insert_with(|| render_glyph_bitmap(c).into()).clone()
    }
}

// extrinsic state (position) stays outside the shared glyph
struct PlacedGlyph { bitmap: Arc<[u8]>, x: f32, y: f32 }
```

`Arc<T>` is Rust's Flyweight vehicle: cloning it shares the underlying allocation instead of duplicating it.

### TypeScript

```typescript
class GlyphCache {
  private cache = new Map<string, Bitmap>();

  get(char: string): Bitmap {
    let bitmap = this.cache.get(char);
    if (!bitmap) {
      bitmap = renderGlyphBitmap(char);
      this.cache.set(char, bitmap);
    }
    return bitmap; // shared reference; treat as read-only
  }
}

interface PlacedGlyph {
  bitmap: Bitmap; // intrinsic, shared
  x: number; y: number; // extrinsic, per-instance
}
```

### Python

```python
from functools import lru_cache

@lru_cache(maxsize=None)
def glyph_bitmap(char: str) -> bytes:
    return render_glyph_bitmap(char)  # cached and shared across calls

class PlacedGlyph:
    def __init__(self, char: str, x: float, y: float) -> None:
        self.bitmap = glyph_bitmap(char)  # shared intrinsic state
        self.x, self.y = x, y  # extrinsic
```

`functools.lru_cache` is an idiomatic Python Flyweight for pure, hashable-input functions.

### Go

```go
type GlyphCache struct {
    mu     sync.Mutex
    glyphs map[rune][]byte
}

func (c *GlyphCache) Get(r rune) []byte {
    c.mu.Lock()
    defer c.mu.Unlock()
    if bitmap, ok := c.glyphs[r]; ok {
        return bitmap
    }
    bitmap := renderGlyphBitmap(r)
    c.glyphs[r] = bitmap
    return bitmap
}
```

Callers must treat the returned slice as read-only since it's shared; document that clearly or return a bounded view type.

### C#

```csharp
public sealed class GlyphCache
{
    private readonly ConcurrentDictionary<char, byte[]> _glyphs = new();

    public byte[] Get(char c) =>
        _glyphs.GetOrAdd(c, ch => RenderGlyphBitmap(ch));
}

public readonly record struct PlacedGlyph(byte[] Bitmap, float X, float Y);
```

### Kotlin

```kotlin
class GlyphCache {
    private val glyphs = ConcurrentHashMap<Char, ByteArray>()

    fun get(c: Char): ByteArray = glyphs.getOrPut(c) { renderGlyphBitmap(c) }
}

data class PlacedGlyph(val bitmap: ByteArray, val x: Float, val y: Float)
```

### C

```c
typedef struct glyph_cache {
    uint8_t *bitmaps[256]; /* indexed by ASCII char, shared intrinsic state */
} glyph_cache_t;

const uint8_t *glyph_cache_get(glyph_cache_t *cache, unsigned char c) {
    if (!cache->bitmaps[c]) {
        cache->bitmaps[c] = render_glyph_bitmap(c);
    }
    return cache->bitmaps[c]; /* caller must not free: cache owns it */
}
```

In C, ownership of the shared data must be documented explicitly — the cache owns every bitmap for its lifetime; callers only borrow a `const` pointer.

### C++

```cpp
class GlyphCache {
public:
    std::shared_ptr<const std::vector<std::byte>> get(char c) {
        std::lock_guard lock(mutex_);
        auto it = glyphs_.find(c);
        if (it != glyphs_.end()) return it->second;
        auto bitmap = std::make_shared<const std::vector<std::byte>>(renderGlyphBitmap(c));
        glyphs_.emplace(c, bitmap);
        return bitmap;
    }
private:
    std::mutex mutex_;
    std::unordered_map<char, std::shared_ptr<const std::vector<std::byte>>> glyphs_;
};
```

`shared_ptr<const T>` both shares the allocation and makes the immutability contract visible in the type.

### Swift

```swift
final class GlyphCache {
    private var glyphs: [Character: Data] = [:]
    private let lock = NSLock()

    func get(_ char: Character) -> Data {
        lock.lock(); defer { lock.unlock() }
        if let bitmap = glyphs[char] { return bitmap }
        let bitmap = renderGlyphBitmap(char)
        glyphs[char] = bitmap
        return bitmap
    }
}

struct PlacedGlyph { let bitmap: Data; let x: Double; let y: Double }
```

`Data` in Swift is itself copy-on-write, so sharing it via the dictionary is already cheap; the cache still avoids repeating the render work.

## Pitfalls

- Adding Flyweight without measurement, then paying complexity cost for no real memory win.
- Mutating "shared" flyweight data in place, corrupting every logical instance that references it.
- Storing extrinsic (per-instance) state inside the shared object by mistake, defeating the whole point.
- Unbounded cache growth with no eviction policy, trading a memory problem for a slow-leak problem.
- Sharing across threads without synchronizing the cache's own insert path (read access to already-cached immutable data is safe; the insert race is not).

## See Also

- [prototype](prototype.md) — the opposite pressure: duplicating state rather than sharing it.
- [singleton](singleton.md) — a Flyweight cache is often itself a singleton-scoped registry; keep injection explicit.
- [object-pool](object-pool.md) — pooling reusable mutable objects, versus sharing immutable ones.
