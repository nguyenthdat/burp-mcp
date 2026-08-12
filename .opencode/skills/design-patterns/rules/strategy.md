# strategy

> Vary an algorithm independently of the code that uses it, by injecting the behavior rather than hard-coding it.

## Intent & Pressure

Reach for Strategy when a piece of behavior (a sort comparator, a pricing rule, a compression algorithm, a retry policy) needs to vary independently of its caller, and multiple interchangeable implementations exist or are expected. The pressure is: the calling code should not need an `if`/`switch` on "which algorithm" scattered through it — it should just call the injected strategy.

Do not reach for a full Strategy interface/trait when a single function or closure parameter does the job — that's the common case. Promote to a named interface/trait only when the strategy needs multiple related methods, holds its own state, or must be selected and stored for later reuse across many calls.

## Native-Construct Alternative

A function/closure parameter (`Fn`, `Callable`, `Func<T>`, function value) is Strategy in its simplest and most common form, and should be the default. Escalate to an interface/trait only when the algorithm needs more than one method or internal configuration/state.

## Language Implementations

### Rust

```rust
fn checkout(cart: &Cart, pricing: impl Fn(&Cart) -> Money) -> Money {
    pricing(cart)
}

// stateless behavior injection via closure/Fn bound — the common case
let total = checkout(&cart, |c| c.subtotal() * 0.9); // 10% off strategy

// named-trait form, for a strategy with multiple methods or stored state
trait PricingStrategy {
    fn price(&self, cart: &Cart) -> Money;
}
```

Prefer a generic `impl Fn` parameter for static dispatch and zero overhead; use `Box<dyn PricingStrategy>` only when the strategy must be chosen at runtime and stored.

### TypeScript

```typescript
type PricingStrategy = (cart: Cart) => Money;

function checkout(cart: Cart, pricing: PricingStrategy): Money {
  return pricing(cart);
}

const total = checkout(cart, (c) => c.subtotal() * 0.9);
```

### Python

```python
from typing import Callable

PricingStrategy = Callable[[Cart], Money]

def checkout(cart: Cart, pricing: PricingStrategy) -> Money:
    return pricing(cart)

total = checkout(cart, lambda c: c.subtotal() * 0.9)
```

### Go

```go
type PricingStrategy func(cart Cart) Money

func Checkout(cart Cart, pricing PricingStrategy) Money {
    return pricing(cart)
}

total := Checkout(cart, func(c Cart) Money { return c.Subtotal() * 0.9 })
```

### C#

```csharp
public static Money Checkout(Cart cart, Func<Cart, Money> pricing) => pricing(cart);

var total = Checkout(cart, c => c.Subtotal() * 0.9m);

// named-interface form for multi-method or stateful strategies:
public interface IPricingStrategy { Money Price(Cart cart); }
```

### Kotlin

```kotlin
fun interface PricingStrategy {
    fun price(cart: Cart): Money
}

fun checkout(cart: Cart, pricing: PricingStrategy): Money = pricing.price(cart)

val total = checkout(cart) { it.subtotal() * 0.9 } // SAM conversion from a lambda
```

### C

```c
typedef double (*pricing_strategy_fn)(const cart_t *cart);

double checkout(const cart_t *cart, pricing_strategy_fn pricing) {
    return pricing(cart);
}

double ten_percent_off(const cart_t *cart) {
    return cart_subtotal(cart) * 0.9;
}
/* usage: checkout(&cart, ten_percent_off); */
```

A plain function pointer is C's Strategy; pass a `void *state` alongside it if the strategy needs configuration.

### C++

```cpp
double checkout(const Cart &cart, const std::function<double(const Cart &)> &pricing) {
    return pricing(cart);
}

auto total = checkout(cart, [](const Cart &c) { return c.subtotal() * 0.9; });

// template form for hot paths, avoiding std::function's type erasure/allocation:
template <typename Pricing>
double checkoutFast(const Cart &cart, Pricing pricing) { return pricing(cart); }
```

### Swift

```swift
typealias PricingStrategy = (Cart) -> Money

func checkout(_ cart: Cart, pricing: PricingStrategy) -> Money {
    pricing(cart)
}

let total = checkout(cart) { $0.subtotal() * 0.9 }
```

## Pitfalls

- Creating a named `Strategy` interface/trait for a single call site with one implementation — a plain function suffices.
- Capturing mutable shared state in a closure/strategy without considering thread-safety when it's invoked concurrently.
- Silently swallowing the strategy's errors instead of propagating a typed failure to the caller.
- Choosing `dyn`/type-erased dispatch for a hot loop where static generic dispatch would avoid real overhead.
- Strategies with hidden side effects that the caller doesn't expect from something named like a pure calculation.

## See Also

- [template-method](template-method.md) — fixing the algorithm skeleton while varying a few steps, versus swapping the whole algorithm.
- [state](state.md) — behavior that varies by current state rather than by injected choice.
- [bridge](bridge.md) — Strategy applied across a whole abstraction rather than a single algorithm call.
