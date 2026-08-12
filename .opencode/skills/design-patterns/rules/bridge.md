# bridge

> Let an abstraction and its implementation vary independently instead of multiplying subclasses for every combination.

## Intent & Pressure

Reach for Bridge when you have two dimensions of variation that would otherwise combine multiplicatively — e.g., "shape" (circle, square) × "renderer" (raster, vector), or "notification type" (alert, digest) × "delivery channel" (email, SMS, push) — and inheritance would force `RasterCircle`, `VectorCircle`, `RasterSquare`, `VectorSquare`, etc. The pressure is genuine independent variation on both axes, each of which is expected to grow.

Do not reach for it when only one axis actually varies, or when both axes are small and closed (two enums covering all combinations may be perfectly clear). Do not introduce Bridge preemptively "in case" a second dimension appears — wait until it does.

## Native-Construct Alternative

If both dimensions are closed, small enums with a lookup/dispatch table are often clearer than an abstraction/implementation class split. Reach for Bridge once either axis is open-ended (pluggable renderers, pluggable channels) or the combination count would otherwise explode.

## Language Implementations

### Rust

```rust
trait Renderer {
    fn render_circle(&self, radius: f64) -> String;
}

struct RasterRenderer;
impl Renderer for RasterRenderer {
    fn render_circle(&self, radius: f64) -> String { format!("raster circle r={radius}") }
}

struct Shape<R: Renderer> {
    renderer: R,
}

impl<R: Renderer> Shape<R> {
    fn draw_circle(&self, radius: f64) -> String {
        self.renderer.render_circle(radius)
    }
}
```

A generic parameter keeps the bridge static (zero-cost) when the renderer is chosen once per call site; use `Box<dyn Renderer>` only if the renderer must change at runtime.

### TypeScript

```typescript
interface Renderer {
  renderCircle(radius: number): string;
}

class RasterRenderer implements Renderer {
  renderCircle(radius: number): string { return `raster circle r=${radius}`; }
}

class Shape {
  constructor(protected renderer: Renderer) {}
}

class Circle extends Shape {
  constructor(renderer: Renderer, private radius: number) { super(renderer); }
  draw(): string { return this.renderer.renderCircle(this.radius); }
}
```

### Python

```python
from typing import Protocol

class Renderer(Protocol):
    def render_circle(self, radius: float) -> str: ...

class RasterRenderer:
    def render_circle(self, radius: float) -> str:
        return f"raster circle r={radius}"

class Circle:
    def __init__(self, renderer: Renderer, radius: float) -> None:
        self._renderer = renderer
        self._radius = radius

    def draw(self) -> str:
        return self._renderer.render_circle(self._radius)
```

### Go

```go
type Renderer interface {
    RenderCircle(radius float64) string
}

type rasterRenderer struct{}
func (rasterRenderer) RenderCircle(radius float64) string {
    return fmt.Sprintf("raster circle r=%v", radius)
}

type Circle struct {
    renderer Renderer
    radius   float64
}

func (c Circle) Draw() string { return c.renderer.RenderCircle(c.radius) }
```

### C#

```csharp
public interface IRenderer
{
    string RenderCircle(double radius);
}

public sealed class RasterRenderer : IRenderer
{
    public string RenderCircle(double radius) => $"raster circle r={radius}";
}

public abstract class Shape
{
    protected Shape(IRenderer renderer) => Renderer = renderer;
    protected IRenderer Renderer { get; }
}

public sealed class Circle : Shape
{
    private readonly double _radius;
    public Circle(IRenderer renderer, double radius) : base(renderer) => _radius = radius;
    public string Draw() => Renderer.RenderCircle(_radius);
}
```

### Kotlin

```kotlin
interface Renderer {
    fun renderCircle(radius: Double): String
}

class RasterRenderer : Renderer {
    override fun renderCircle(radius: Double) = "raster circle r=$radius"
}

class Circle(private val renderer: Renderer, private val radius: Double) {
    fun draw(): String = renderer.renderCircle(radius)
}
```

### C

```c
typedef struct renderer {
    char *(*render_circle)(struct renderer *self, double radius);
} renderer_t;

char *raster_render_circle(renderer_t *self, double radius) {
    char *buf = malloc(64);
    snprintf(buf, 64, "raster circle r=%g", radius);
    return buf;
}

typedef struct circle {
    renderer_t *renderer; /* bridge: circle does not know which renderer */
    double radius;
} circle_t;

char *circle_draw(circle_t *c) {
    return c->renderer->render_circle(c->renderer, c->radius);
}
```

### C++

```cpp
class Renderer {
public:
    virtual ~Renderer() = default;
    virtual std::string renderCircle(double radius) const = 0;
};

class RasterRenderer : public Renderer {
public:
    std::string renderCircle(double radius) const override {
        return std::format("raster circle r={}", radius);
    }
};

class Circle {
public:
    Circle(const Renderer &renderer, double radius) : renderer_(renderer), radius_(radius) {}
    std::string draw() const { return renderer_.renderCircle(radius_); }
private:
    const Renderer &renderer_;
    double radius_;
};
```

### Swift

```swift
protocol Renderer {
    func renderCircle(radius: Double) -> String
}

struct RasterRenderer: Renderer {
    func renderCircle(radius: Double) -> String { "raster circle r=\(radius)" }
}

struct Circle {
    let renderer: Renderer
    let radius: Double
    func draw() -> String { renderer.renderCircle(radius: radius) }
}
```

## Pitfalls

- Building a Bridge when only one axis actually varies — that's just Strategy with extra ceremony.
- Letting the abstraction reach into implementation-specific details, defeating the independence the pattern is meant to provide.
- Confusing Bridge with Adapter: Bridge is designed in upfront for two independent axes; Adapter retrofits compatibility after the fact.
- Over-generalizing a bridge for a second dimension that never actually gets a second implementation.

## See Also

- [adapter](adapter.md) — retrofitting compatibility versus designing in independent variation.
- [strategy](strategy.md) — Bridge is often "Strategy applied to a whole abstraction," not just one algorithm.
- [abstract-factory](abstract-factory.md) — often used to construct the correct abstraction/implementation pairing.
