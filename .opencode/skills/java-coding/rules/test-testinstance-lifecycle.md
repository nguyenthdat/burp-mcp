# test-testinstance-lifecycle

> Choose `@TestInstance` lifecycle deliberately

## Why It Matters

JUnit 5 defaults to `PER_METHOD` lifecycle: a fresh test class instance is created for every `@Test`, so instance fields never leak state between tests. Switching to `PER_CLASS` (one instance shared across all tests) enables non-static `@BeforeAll` and shared expensive fixtures, but it reintroduces the exact cross-test state leakage JUnit 4 developers spent years working around if fields aren't reset carefully. Pick the lifecycle based on whether tests need isolation or need to share costly setup, not by default.

## Bad

```java
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

import java.util.ArrayList;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

@TestInstance(TestInstance.Lifecycle.PER_CLASS) // chosen without thinking about shared mutable state
class OrderQueueTest {

    private final List<Order> processedOrders = new ArrayList<>(); // shared across every test!

    @BeforeAll
    void seedQueue() {
        processedOrders.add(new Order("seed"));
    }

    @Test
    void firstTestAddsOneOrder() {
        processedOrders.add(new Order("A"));
        assertThat(processedOrders).hasSize(2); // passes alone, fails when run after another test
    }

    @Test
    void secondTestAddsOneOrder() {
        processedOrders.add(new Order("B"));
        assertThat(processedOrders).hasSize(2); // actually 3 by now due to leaked state
    }
}
```

## Good

```java
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

// default PER_METHOD lifecycle: a new instance per test, no leakage possible
class OrderQueueTest {

    private List<Order> processedOrders;

    @BeforeEach
    void freshQueue() {
        processedOrders = new ArrayList<>();
    }

    @Test
    void firstTestAddsOneOrder() {
        processedOrders.add(new Order("A"));
        assertThat(processedOrders).hasSize(1);
    }

    @Test
    void secondTestAddsOneOrder() {
        processedOrders.add(new Order("B"));
        assertThat(processedOrders).hasSize(1);
    }
}
```

## When PER_CLASS Is Acceptable

`PER_CLASS` is a reasonable choice when the fixture is expensive, read-only, and genuinely shared (e.g., a Testcontainers database or a loaded reference dataset), and no test mutates it.

```java
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestInstance;

import static org.assertj.core.api.Assertions.assertThat;

@TestInstance(TestInstance.Lifecycle.PER_CLASS)
class ExchangeRateTableTest {

    private ExchangeRateTable rates; // read-only after setup, safe to share

    @BeforeAll
    void loadRatesOnce() { // non-static @BeforeAll is only legal under PER_CLASS
        rates = ExchangeRateTable.loadFromFile("test-rates.csv"); // expensive I/O, done once
    }

    @Test
    void convertsUsdToEur() {
        assertThat(rates.convert(100, "USD", "EUR")).isEqualTo(92.30);
    }

    @Test
    void convertsUsdToJpy() {
        assertThat(rates.convert(100, "USD", "JPY")).isEqualTo(14_820.00);
    }
}
```

## See Also

- [`test-nested-grouping`](test-nested-grouping.md) - how lifecycle choice interacts with `@Nested` classes
- [`conc-avoid-shared-mutable-state`](conc-avoid-shared-mutable-state.md) - the general hazard of shared mutable fields
- [`test-integration-test-separation`](test-integration-test-separation.md) - when shared expensive fixtures belong in integration tests instead
