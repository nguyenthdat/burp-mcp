# iterator

> Traverse a collection's elements sequentially without exposing its internal representation.

## Intent & Pressure

Reach for a custom Iterator when a collection has a representation-specific traversal order (a tree in depth-first order, a graph in topological order, a paginated remote API) that the language's built-in iteration protocol doesn't already give you for free. The pressure is decoupling "how to traverse" from "what the caller does with each element," while composing with the rest of the ecosystem's iteration tools (`for` loops, comprehensions, `zip`/`map`/`filter`).

Do not reach for a hand-rolled traversal interface when the language's standard iteration protocol (`Iterator`/`IntoIterator` in Rust, generators/`Iterable` in TypeScript/Python, `range`-over-func in Go, `IEnumerable<T>` in C#, `Sequence`/`Iterator` in Kotlin, `Sequence` in Swift) already covers the case — almost always the right first move.

## Native-Construct Alternative

Implement the language's standard iteration protocol directly rather than inventing a parallel one. This is nearly always sufficient and gives you `for`-loop, comprehension, and standard-library-combinator support for free.

## Language Implementations

### Rust

```rust
struct Fibonacci { curr: u64, next: u64 }

impl Iterator for Fibonacci {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let value = self.curr;
        self.curr = self.next;
        self.next = value + self.next;
        Some(value)
    }
}

// usage: for n in Fibonacci { curr: 0, next: 1 }.take(10) { ... }
```

Implementing `Iterator` gives access to every adapter (`take`, `map`, `zip`, `filter`) without writing them yourself.

### TypeScript

```typescript
class Fibonacci implements Iterable<number> {
  [Symbol.iterator](): Iterator<number> {
    let [curr, next] = [0, 1];
    return {
      next: () => {
        const value = curr;
        [curr, next] = [next, curr + next];
        return { value, done: false };
      },
    };
  }
}

// usage: for (const n of new Fibonacci()) { if (n > 100) break; ... }
```

A generator function is usually simpler than a manual `Iterator` object:

```typescript
function* fibonacci(): Generator<number> {
  let [curr, next] = [0, 1];
  while (true) {
    yield curr;
    [curr, next] = [next, curr + next];
  }
}
```

### Python

```python
def fibonacci():
    curr, nxt = 0, 1
    while True:
        yield curr
        curr, nxt = nxt, curr + nxt

# usage: for n in itertools.islice(fibonacci(), 10): ...
```

A generator function is the idiomatic Python Iterator — implementing `__iter__`/`__next__` manually is rarely needed.

### Go

```go
func Fibonacci(yield func(uint64) bool) {
    a, b := uint64(0), uint64(1)
    for yield(a) {
        a, b = b, a+b
    }
}

// usage (Go 1.23+ range-over-func): for n := range Fibonacci { if n > 100 { break }; ... }
```

### C#

```csharp
public sealed class Fibonacci : IEnumerable<ulong>
{
    public IEnumerator<ulong> GetEnumerator()
    {
        ulong curr = 0, next = 1;
        while (true)
        {
            yield return curr;
            (curr, next) = (next, curr + next);
        }
    }
    IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();
}
```

C#'s `yield return` inside `GetEnumerator()` compiles to the full state machine automatically — no manual `MoveNext`/`Current` bookkeeping needed.

### Kotlin

```kotlin
fun fibonacci(): Sequence<Long> = sequence {
    var curr = 0L
    var next = 1L
    while (true) {
        yield(curr)
        val newNext = curr + next
        curr = next
        next = newNext
    }
}

// usage: fibonacci().take(10).forEach { ... }
```

### C

```c
typedef struct fib_iter { uint64_t curr, next; } fib_iter_t;

void fib_iter_init(fib_iter_t *it) { it->curr = 0; it->next = 1; }

uint64_t fib_iter_next(fib_iter_t *it) {
    uint64_t value = it->curr;
    uint64_t new_next = it->curr + it->next;
    it->curr = it->next;
    it->next = new_next;
    return value;
}
/* caller controls the loop bound explicitly: C has no built-in "done" signal here */
```

C has no iteration protocol, so a struct plus a `_next` function (with an explicit sentinel or out-parameter for "done") is the idiomatic form.

### C++

```cpp
class FibonacciIterator {
public:
    using value_type = std::uint64_t;
    std::uint64_t operator*() const { return curr_; }
    FibonacciIterator &operator++() {
        std::uint64_t newNext = curr_ + next_;
        curr_ = next_;
        next_ = newNext;
        return *this;
    }
    bool operator!=(std::default_sentinel_t) const { return true; } // infinite
private:
    std::uint64_t curr_ = 0, next_ = 1;
};
```

Implementing the standard iterator operators (`*`, `++`, comparison) lets range-based `for` and `<ranges>` algorithms use it directly.

### Swift

```swift
struct FibonacciSequence: Sequence, IteratorProtocol {
    private var curr = 0, next = 1
    mutating func next() -> Int? {
        defer { (curr, next) = (next, curr + next) }
        return curr
    }
}

// usage: for n in FibonacciSequence().prefix(10) { ... }
```

Conforming to `Sequence`/`IteratorProtocol` gives access to every `Sequence` combinator (`prefix`, `map`, `filter`) for free.

## Pitfalls

- Inventing a bespoke traversal interface instead of implementing the language's standard iteration protocol, losing compatibility with `for`-loops and standard combinators.
- Iterators that mutate the underlying collection during traversal without a documented invalidation contract.
- Infinite iterators consumed without a `take`/`prefix`/bound, hanging the caller.
- Iterator state that isn't reset correctly when reused, silently resuming from a stale position.
- Ignoring `ExactSizeIterator`/`hasNext`-style size hints when they'd let callers preallocate.

## See Also

- [composite](composite.md) — a frequent target for custom iteration order over recursive structures.
- [visitor](visitor.md) — running an operation over every element, versus handing elements to the caller one at a time.
- [generator/coroutine idioms](strategy.md) — many languages implement Iterator via the same generator machinery used for cooperative strategies.
