# interpreter

> Represent sentences in a small, stable grammar as an object tree, and evaluate that tree repeatedly.

## Intent & Pressure

Reach for Interpreter when you have a small, genuinely stable grammar that must be parsed once and evaluated many times with different inputs or contexts — a filter/query expression language, a rule engine, a simple calculator embedded in a product, feature-flag targeting rules. The pressure is repeated evaluation of structurally similar expressions where hand-written `if`/`else` chains would duplicate parsing and evaluation logic across call sites.

Do not reach for it for a genuinely complex language (a real programming language, a full query language) — use an existing parser generator or an established expression library instead of hand-rolling a grammar and evaluator. Do not reach for it when a fixed, small set of predicates can just be composed with plain functions (see [specification](specification.md)) — Interpreter earns its cost when the grammar itself needs to be *data* (parsed from text/config), not just composed in code.

## Native-Construct Alternative

Prefer an existing expression/rule/query library for your language before writing a bespoke grammar and evaluator. If the "rules" are actually just composable predicates authored in code rather than parsed from text, use [specification](specification.md) instead — it's simpler and type-checked.

## Language Implementations

### Rust

```rust
enum Expr {
    Number(f64),
    Add(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Variable(String),
}

fn eval(expr: &Expr, ctx: &HashMap<String, f64>) -> Result<f64, EvalError> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Add(l, r) => Ok(eval(l, ctx)? + eval(r, ctx)?),
        Expr::Multiply(l, r) => Ok(eval(l, ctx)? * eval(r, ctx)?),
        Expr::Variable(name) => ctx.get(name).copied().ok_or_else(|| EvalError::UnknownVariable(name.clone())),
    }
}
```

### TypeScript

```typescript
type Expr =
  | { kind: "number"; value: number }
  | { kind: "add"; left: Expr; right: Expr }
  | { kind: "multiply"; left: Expr; right: Expr }
  | { kind: "variable"; name: string };

function evaluate(expr: Expr, ctx: Record<string, number>): number {
  switch (expr.kind) {
    case "number": return expr.value;
    case "add": return evaluate(expr.left, ctx) + evaluate(expr.right, ctx);
    case "multiply": return evaluate(expr.left, ctx) * evaluate(expr.right, ctx);
    case "variable": {
      const value = ctx[expr.name];
      if (value === undefined) throw new Error(`unknown variable: ${expr.name}`);
      return value;
    }
  }
}
```

### Python

```python
from dataclasses import dataclass

class Expr: ...

@dataclass
class Number(Expr):
    value: float

@dataclass
class Add(Expr):
    left: Expr
    right: Expr

@dataclass
class Variable(Expr):
    name: str

def evaluate(expr: Expr, ctx: dict[str, float]) -> float:
    match expr:
        case Number(value):
            return value
        case Add(left, right):
            return evaluate(left, ctx) + evaluate(right, ctx)
        case Variable(name):
            if name not in ctx:
                raise KeyError(f"unknown variable: {name}")
            return ctx[name]
        case _:
            raise TypeError(f"unhandled expression: {expr!r}")
```

### Go

```go
type Expr interface{ Eval(ctx map[string]float64) (float64, error) }

type Number struct{ Value float64 }
func (n Number) Eval(ctx map[string]float64) (float64, error) { return n.Value, nil }

type Add struct{ Left, Right Expr }
func (a Add) Eval(ctx map[string]float64) (float64, error) {
    l, err := a.Left.Eval(ctx)
    if err != nil { return 0, err }
    r, err := a.Right.Eval(ctx)
    if err != nil { return 0, err }
    return l + r, nil
}

type Variable struct{ Name string }
func (v Variable) Eval(ctx map[string]float64) (float64, error) {
    val, ok := ctx[v.Name]
    if !ok { return 0, fmt.Errorf("unknown variable: %s", v.Name) }
    return val, nil
}
```

### C#

```csharp
public abstract record Expr;
public sealed record Number(double Value) : Expr;
public sealed record Add(Expr Left, Expr Right) : Expr;
public sealed record Variable(string Name) : Expr;

public static double Evaluate(Expr expr, IReadOnlyDictionary<string, double> ctx) => expr switch
{
    Number n => n.Value,
    Add a => Evaluate(a.Left, ctx) + Evaluate(a.Right, ctx),
    Variable v => ctx.TryGetValue(v.Name, out var value)
        ? value
        : throw new KeyNotFoundException($"unknown variable: {v.Name}"),
    _ => throw new NotSupportedException(),
};
```

### Kotlin

```kotlin
sealed interface Expr
data class Number(val value: Double) : Expr
data class Add(val left: Expr, val right: Expr) : Expr
data class Variable(val name: String) : Expr

fun evaluate(expr: Expr, ctx: Map<String, Double>): Double = when (expr) {
    is Number -> expr.value
    is Add -> evaluate(expr.left, ctx) + evaluate(expr.right, ctx)
    is Variable -> ctx[expr.name] ?: error("unknown variable: ${expr.name}")
}
```

### C

```c
typedef enum { EXPR_NUMBER, EXPR_ADD, EXPR_VARIABLE } expr_kind_t;

typedef struct expr {
    expr_kind_t kind;
    union {
        double number;
        struct { struct expr *left, *right; } add;
        char *variable;
    } as;
} expr_t;

int expr_eval(const expr_t *e, const hash_map_t *ctx, double *out) {
    switch (e->kind) {
        case EXPR_NUMBER:
            *out = e->as.number;
            return 0;
        case EXPR_ADD: {
            double l, r;
            if (expr_eval(e->as.add.left, ctx, &l) != 0) return -1;
            if (expr_eval(e->as.add.right, ctx, &r) != 0) return -1;
            *out = l + r;
            return 0;
        }
        case EXPR_VARIABLE: {
            double *value = hash_map_get(ctx, e->as.variable);
            if (!value) return -1;
            *out = *value;
            return 0;
        }
    }
    return -1;
}
```

### C++

```cpp
struct Expr {
    virtual ~Expr() = default;
    virtual double eval(const std::map<std::string, double> &ctx) const = 0;
};

struct Number : Expr {
    explicit Number(double v) : value(v) {}
    double eval(const std::map<std::string, double> &) const override { return value; }
    double value;
};

struct Add : Expr {
    Add(std::unique_ptr<Expr> l, std::unique_ptr<Expr> r) : left(std::move(l)), right(std::move(r)) {}
    double eval(const std::map<std::string, double> &ctx) const override {
        return left->eval(ctx) + right->eval(ctx);
    }
    std::unique_ptr<Expr> left, right;
};
```

### Swift

```swift
indirect enum Expr {
    case number(Double)
    case add(Expr, Expr)
    case variable(String)
}

func evaluate(_ expr: Expr, context: [String: Double]) throws -> Double {
    switch expr {
    case .number(let value): return value
    case .add(let left, let right): return try evaluate(left, context: context) + evaluate(right, context: context)
    case .variable(let name):
        guard let value = context[name] else { throw EvalError.unknownVariable(name) }
        return value
    }
}
```

## Pitfalls

- Growing the "small grammar" into a real programming language over time instead of adopting an existing parser/expression library once complexity crosses that line.
- Unbounded recursion depth on user-supplied expressions, enabling a stack-overflow denial of service.
- Evaluating untrusted expressions without sandboxing (no filesystem/network access, bounded loops, bounded recursion).
- No caching/reuse of parsed trees when the same expression is evaluated repeatedly with different contexts.

## See Also

- [specification](specification.md) — composable predicates authored in code, not parsed from a grammar.
- [visitor](visitor.md) — adding new operations (e.g., pretty-printing, optimization) over the expression tree without changing each node type.
- [composite](composite.md) — the recursive tree structure Interpreter's expressions typically use.
