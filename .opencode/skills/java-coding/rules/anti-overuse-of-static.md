# anti-overuse-of-static

> Don't overuse `static` methods and fields

## Why It Matters

Static methods can't be mocked or substituted through normal dependency injection, so overusing them forces tests to either exercise real I/O, use brittle static-mocking frameworks, or skip testing that code path entirely. Static mutable fields are effectively global variables shared across every thread and every test in the JVM, which is exactly the kind of hidden shared state that produces flaky, order-dependent test failures.

## Bad

```java
public class PriceCalculator {
  private static double taxRate = 0.08; // Shared mutable global state

  public static void setTaxRate(double rate) { // Any code, anywhere, can change this
    taxRate = rate;
  }

  public static BigDecimal calculateTotal(BigDecimal price) {
    return price.multiply(BigDecimal.valueOf(1 + taxRate));
  }
}

// Test A sets taxRate to 0.20 for its scenario...
PriceCalculator.setTaxRate(0.20);
// ...and Test B, running concurrently or afterward, silently inherits it
assertEquals(new BigDecimal("108.00"), PriceCalculator.calculateTotal(new BigDecimal("100")));
// Fails intermittently depending on test execution order
```

## Good

```java
public final class PriceCalculator {
  private final BigDecimal taxRate;

  public PriceCalculator(BigDecimal taxRate) {
    this.taxRate = taxRate;
  }

  public BigDecimal calculateTotal(BigDecimal price) {
    return price.multiply(BigDecimal.ONE.add(taxRate));
  }
}

// Each test constructs its own isolated instance - no shared state, no ordering issues
@Test
void appliesConfiguredTaxRate() {
  PriceCalculator calculator = new PriceCalculator(new BigDecimal("0.20"));
  assertEquals(new BigDecimal("120.00"), calculator.calculateTotal(new BigDecimal("100")));
}
```

## When static Is the Right Call

```java
// Pure, stateless functions with no dependencies are perfectly fine as static -
// they're deterministic and trivially testable without instantiation.
public final class MathUtils {
  private MathUtils() { } // Prevent instantiation of a pure-function holder

  public static int clamp(int value, int min, int max) {
    return Math.max(min, Math.min(max, value));
  }
}

// Immutable static final constants are also fine.
public static final int DEFAULT_PAGE_SIZE = 20;
```

## See Also

- [`anti-singleton-static-abuse`](anti-singleton-static-abuse.md) - The specific singleton variant of this same problem
- [`anti-god-class`](anti-god-class.md) - Static-utility classes often grow into God classes over time
- [`conc-avoid-shared-mutable-state`](conc-avoid-shared-mutable-state.md) - Why shared mutable state (static or not) causes concurrency bugs
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - A legitimate, stateless use of static methods
