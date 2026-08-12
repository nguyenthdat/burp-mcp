# test-avoid-logic-in-tests

> Avoid conditionals/loops inside test bodies

## Why It Matters

A test with an `if` or a `for` loop has branches, and untested branches in test code are bugs waiting to hide production bugs. Logic in a test also means the test itself can have defects, and a failure no longer points at one clear expectation — the reader has to mentally execute the loop to figure out which iteration failed. Parameterized tests and plain, linear assertions eliminate the need for control flow entirely.

## Bad

```java
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

class InventoryTest {

    @Test
    void allItemsHaveNonNegativeStock() {
        List<Item> items = InventoryFixture.sample();

        for (Item item : items) { // loop hides which item actually failed
            if (item.category() == Category.DISCONTINUED) {
                continue; // conditional skip logic buried in the test
            }
            assertThat(item.stock()).isGreaterThanOrEqualTo(0);
        }
    }

    @Test
    void discountAppliesCorrectly() {
        Order order = OrderFixture.sample();
        double expected;
        if (order.total() > 100) { // computing the expected value with the same logic under test
            expected = order.total() * 0.9;
        } else {
            expected = order.total();
        }
        assertThat(order.totalAfterDiscount()).isEqualTo(expected);
    }
}
```

## Good

```java
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

import java.util.stream.Stream;

import static org.assertj.core.api.Assertions.assertThat;

class InventoryTest {

    @ParameterizedTest
    @MethodSource("activeItems")
    void activeItemsHaveNonNegativeStock(Item item) {
        assertThat(item.stock()).isGreaterThanOrEqualTo(0);
    }

    static Stream<Item> activeItems() {
        return InventoryFixture.sample().stream()
            .filter(item -> item.category() != Category.DISCONTINUED);
    }

    @org.junit.jupiter.params.ParameterizedTest
    @org.junit.jupiter.params.provider.CsvSource({
        "150.00, 135.00", // over threshold: discount applies
        "80.00,  80.00"   // under threshold: no discount
    })
    void discountAppliesOnlyAboveThreshold(double total, double expectedAfterDiscount) {
        Order order = Order.withTotal(total);

        assertThat(order.totalAfterDiscount()).isEqualTo(expectedAfterDiscount);
    }
}
```

## Preferring AssertJ Collection Assertions Over Manual Loops

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class ReportTest {

    @Test
    void everyLineItemHasAPositiveAmount() {
        Report report = ReportFixture.sample();

        // AssertJ expresses "for all elements" without a loop or a hidden branch
        assertThat(report.lineItems())
            .allSatisfy(item -> assertThat(item.amount()).isPositive());
    }
}
```

## See Also

- [`test-parameterized-tests`](test-parameterized-tests.md) - the correct replacement for loops over test cases
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - conditionals are often a sign a test covers more than one concept
- [`coll-stream-vs-loop`](coll-stream-vs-loop.md) - general guidance on loops vs. declarative alternatives
