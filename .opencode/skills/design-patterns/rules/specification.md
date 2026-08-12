# specification

> Encapsulate a business predicate as a first-class, named, composable value instead of an inline boolean expression.

## Intent & Pressure

Reach for Specification when a business rule needs to be named, unit-tested independently, reused in more than one place (filtering a list, validating a single entity, generating a query), and composed with other rules via AND/OR/NOT (`IsActiveCustomer.and(HasOutstandingBalance)`). The pressure is rule sprawl: the same "is this customer eligible" logic duplicated (and drifting) across a filter, a validator, and a query builder.

Do not reach for it when a predicate is used in exactly one place and isn't composed with anything else — a plain boolean function or lambda inline is simpler. Specification earns its cost specifically when predicates need naming, reuse, and composition as first-class values.

## Native-Construct Alternative

A plain function or lambda returning `bool` is a full specification for the single-use case. Promote to a dedicated composable type once you need AND/OR/NOT composition or reuse across multiple contexts (in-memory filtering and query generation both needing "the same" rule).

## Language Implementations

### Rust

```rust
trait Specification<T> {
    fn is_satisfied_by(&self, candidate: &T) -> bool;
    fn and<S: Specification<T>>(self, other: S) -> AndSpec<Self, S> where Self: Sized {
        AndSpec(self, other)
    }
}

struct AndSpec<A, B>(A, B);
impl<T, A: Specification<T>, B: Specification<T>> Specification<T> for AndSpec<A, B> {
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.0.is_satisfied_by(candidate) && self.1.is_satisfied_by(candidate)
    }
}

struct IsActiveCustomer;
impl Specification<Customer> for IsActiveCustomer {
    fn is_satisfied_by(&self, c: &Customer) -> bool { c.status == Status::Active }
}
```

### TypeScript

```typescript
interface Specification<T> {
  isSatisfiedBy(candidate: T): boolean;
}

class AndSpec<T> implements Specification<T> {
  constructor(private a: Specification<T>, private b: Specification<T>) {}
  isSatisfiedBy(candidate: T): boolean {
    return this.a.isSatisfiedBy(candidate) && this.b.isSatisfiedBy(candidate);
  }
}

class IsActiveCustomer implements Specification<Customer> {
  isSatisfiedBy(customer: Customer): boolean { return customer.status === "active"; }
}

// usage: customers.filter((c) => new IsActiveCustomer().isSatisfiedBy(c));
```

### Python

```python
from typing import Protocol, Callable

class Specification(Protocol[T]):
    def is_satisfied_by(self, candidate: T) -> bool: ...

def and_spec(a: Specification[T], b: Specification[T]) -> Callable[[T], bool]:
    return lambda candidate: a.is_satisfied_by(candidate) and b.is_satisfied_by(candidate)

class IsActiveCustomer:
    def is_satisfied_by(self, customer: Customer) -> bool:
        return customer.status == "active"
```

### Go

```go
type Specification[T any] func(candidate T) bool

func And[T any](a, b Specification[T]) Specification[T] {
    return func(candidate T) bool { return a(candidate) && b(candidate) }
}

func IsActiveCustomer(c Customer) bool { return c.Status == StatusActive }
```

Go's generics plus a function type make Specification a plain composable function value — no interface hierarchy needed.

### C#

```csharp
public interface ISpecification<T>
{
    bool IsSatisfiedBy(T candidate);
}

public sealed class AndSpecification<T> : ISpecification<T>
{
    private readonly ISpecification<T> _a, _b;
    public AndSpecification(ISpecification<T> a, ISpecification<T> b) { _a = a; _b = b; }
    public bool IsSatisfiedBy(T candidate) => _a.IsSatisfiedBy(candidate) && _b.IsSatisfiedBy(candidate);
}

public sealed class IsActiveCustomer : ISpecification<Customer>
{
    public bool IsSatisfiedBy(Customer c) => c.Status == CustomerStatus.Active;
}
```

### Kotlin

```kotlin
fun interface Specification<T> {
    fun isSatisfiedBy(candidate: T): Boolean

    infix fun and(other: Specification<T>): Specification<T> =
        Specification { isSatisfiedBy(it) && other.isSatisfiedBy(it) }
}

val isActiveCustomer = Specification<Customer> { it.status == Status.ACTIVE }
```

### C

```c
typedef bool (*spec_fn)(const void *candidate);

typedef struct { spec_fn a, b; } and_spec_state_t;

bool and_spec_apply(const and_spec_state_t *state, const void *candidate) {
    return state->a(candidate) && state->b(candidate);
}

bool is_active_customer(const void *candidate) {
    const customer_t *c = candidate;
    return c->status == STATUS_ACTIVE;
}
```

### C++

```cpp
template <typename T>
class Specification {
public:
    virtual ~Specification() = default;
    virtual bool isSatisfiedBy(const T &candidate) const = 0;
};

template <typename T>
class AndSpecification : public Specification<T> {
public:
    AndSpecification(std::shared_ptr<Specification<T>> a, std::shared_ptr<Specification<T>> b)
        : a_(std::move(a)), b_(std::move(b)) {}
    bool isSatisfiedBy(const T &candidate) const override {
        return a_->isSatisfiedBy(candidate) && b_->isSatisfiedBy(candidate);
    }
private:
    std::shared_ptr<Specification<T>> a_, b_;
};
```

### Swift

```swift
protocol Specification {
    associatedtype Candidate
    func isSatisfied(by candidate: Candidate) -> Bool
}

struct AndSpecification<A: Specification, B: Specification>: Specification where A.Candidate == B.Candidate {
    let a: A
    let b: B
    func isSatisfied(by candidate: A.Candidate) -> Bool {
        a.isSatisfied(by: candidate) && b.isSatisfied(by: candidate)
    }
}

struct IsActiveCustomer: Specification {
    func isSatisfied(by candidate: Customer) -> Bool { candidate.status == .active }
}
```

## Pitfalls

- Building composable Specification objects for a single-use predicate that never gets reused or combined — a plain function is simpler.
- Two divergent implementations of "the same" rule (one for in-memory filtering, one for a database query) drifting apart over time — keep the intent documented and tested identically even if the mechanism differs.
- Deeply nested AND/OR/NOT trees that become harder to read than the equivalent boolean expression would have been.
- Specifications with hidden side effects (a "specification" that also logs or mutates), breaking the assumption that evaluating one is safe and free of side effects.

## See Also

- [strategy](strategy.md) — Specification is a strategy specialized to boolean predicates with composition operators.
- [interpreter](interpreter.md) — when rules must be parsed from data/config rather than composed in code.
- [null-object](null-object.md) — trivial always-true/always-false specifications follow the same no-op-implementation idea.
