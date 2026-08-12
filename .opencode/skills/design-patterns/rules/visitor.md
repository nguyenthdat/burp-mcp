# visitor

> Add new operations over a stable set of element types without modifying those types, by moving the operation into a separate visitor.

## Intent & Pressure

Reach for Visitor when you have a fixed, rarely-changing set of element/node types (an AST's expression kinds, a document's block types) but need to add many independently evolving operations over them (pretty-printing, type-checking, optimization, serialization) without editing every element type each time. The pressure is: operations should be addable without touching the element hierarchy, because the element set is stable but the operation set is not.

Do not reach for it when the situation is reversed — the element set changes often but operations are stable. In that case, an `enum`/sealed-class with exhaustive `match`/`when` is simpler: the compiler forces you to update every match arm when a new element is added, which is exactly the safety net you want when elements change more than operations.

## Native-Construct Alternative

If the language has closed sum types with exhaustiveness checking (`enum`+`match`, `sealed class`+`when`, `std::variant`+`std::visit`), a plain function with an exhaustive match *is* effectively Visitor without a separate class hierarchy — reach for the classic double-dispatch `accept`/`visit` object structure only in languages/situations lacking that (open, extensible element hierarchies via inheritance).

## Language Implementations

### Rust

```rust
enum Expr { Number(f64), Add(Box<Expr>, Box<Expr>) }

// "visitor" is just an exhaustive match — new operations are new functions
fn pretty_print(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => n.to_string(),
        Expr::Add(l, r) => format!("({} + {})", pretty_print(l), pretty_print(r)),
    }
}

fn evaluate(expr: &Expr) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Add(l, r) => evaluate(l) + evaluate(r),
    }
}
```

### TypeScript

```typescript
type Expr =
  | { kind: "number"; value: number }
  | { kind: "add"; left: Expr; right: Expr };

function prettyPrint(expr: Expr): string {
  switch (expr.kind) {
    case "number": return String(expr.value);
    case "add": return `(${prettyPrint(expr.left)} + ${prettyPrint(expr.right)})`;
  }
}
```

### Python

```python
def pretty_print(expr: Expr) -> str:
    match expr:
        case Number(value):
            return str(value)
        case Add(left, right):
            return f"({pretty_print(left)} + {pretty_print(right)})"
        case _:
            raise TypeError(f"unhandled expr: {expr!r}")
```

### Go

```go
type Visitor interface {
    VisitNumber(n Number)
    VisitAdd(a Add)
}

type Expr interface{ Accept(v Visitor) }

func (n Number) Accept(v Visitor) { v.VisitNumber(n) }
func (a Add) Accept(v Visitor)    { v.VisitAdd(a) }

type PrettyPrinter struct{ Result string }
func (p *PrettyPrinter) VisitNumber(n Number) { p.Result = fmt.Sprintf("%v", n.Value) }
func (p *PrettyPrinter) VisitAdd(a Add) {
    var left, right PrettyPrinter
    a.Left.Accept(&left)
    a.Right.Accept(&right)
    p.Result = fmt.Sprintf("(%s + %s)", left.Result, right.Result)
}
```

Go's interfaces are open (any type can implement `Expr`), so a real `accept`/`Visitor` double-dispatch pair is more common here than in Rust/Kotlin, where closed sum types make a plain type switch just as safe.

### C#

```csharp
public abstract record Expr;
public sealed record Number(double Value) : Expr;
public sealed record Add(Expr Left, Expr Right) : Expr;

public static string PrettyPrint(Expr expr) => expr switch
{
    Number n => n.Value.ToString(),
    Add a => $"({PrettyPrint(a.Left)} + {PrettyPrint(a.Right)})",
    _ => throw new NotSupportedException(),
};
```

### Kotlin

```kotlin
sealed interface Expr
data class Number(val value: Double) : Expr
data class Add(val left: Expr, val right: Expr) : Expr

fun prettyPrint(expr: Expr): String = when (expr) {
    is Number -> expr.value.toString()
    is Add -> "(${prettyPrint(expr.left)} + ${prettyPrint(expr.right)})"
}
```

### C

```c
typedef enum { EXPR_NUMBER, EXPR_ADD } expr_kind_t;
typedef struct expr {
    expr_kind_t kind;
    union {
        double number;
        struct { struct expr *left, *right; } add;
    } as;
} expr_t;

void pretty_print(const expr_t *e, char *out, size_t out_len) {
    switch (e->kind) {
        case EXPR_NUMBER:
            snprintf(out, out_len, "%g", e->as.number);
            break;
        case EXPR_ADD: {
            char left[64], right[64];
            pretty_print(e->as.add.left, left, sizeof(left));
            pretty_print(e->as.add.right, right, sizeof(right));
            snprintf(out, out_len, "(%s + %s)", left, right);
            break;
        }
    }
}
```

### C++

```cpp
struct Number { double value; };
struct Add;
using Expr = std::variant<Number, std::unique_ptr<Add>>; // simplified; real ASTs box recursively

std::string prettyPrint(const Expr &expr) {
    return std::visit(overloaded{
        [](const Number &n) { return std::to_string(n.value); },
        [](const std::unique_ptr<Add> &a) { return std::string("(add)"); }, // recurse in practice
    }, expr);
}
```

`std::variant` + `std::visit` (with an `overloaded` helper) is C++'s closed-set Visitor without a class hierarchy; use the classic virtual `accept`/`Visitor` pair only for an open, inheritance-based element hierarchy.

### Swift

```swift
indirect enum Expr {
    case number(Double)
    case add(Expr, Expr)
}

func prettyPrint(_ expr: Expr) -> String {
    switch expr {
    case .number(let value): return "\(value)"
    case .add(let left, let right): return "(\(prettyPrint(left)) + \(prettyPrint(right)))"
    }
}
```

## Pitfalls

- Applying Visitor to an element set that changes frequently — every new element requires updating every visitor, which is the opposite of the intended stability trade-off.
- Reaching for a full `accept`/`Visitor` double-dispatch hierarchy in a language with closed sum types and exhaustive matching, when a plain function would give the same safety with less ceremony.
- Visitors that accumulate mutable state across a traversal without documenting ordering guarantees (pre-order vs. post-order).
- Missing a case in an open/inheritance-based visitor hierarchy silently, because the language doesn't enforce exhaustiveness the way `match`/`when`/`switch` on a closed type does.

## See Also

- [composite](composite.md) — Visitor most often operates over a Composite's recursive node structure.
- [interpreter](interpreter.md) — Visitor is a natural way to add operations (eval, print, optimize) over an Interpreter's expression tree.
- [iterator](iterator.md) — traversal mechanics that Visitor's dispatch typically relies on.
