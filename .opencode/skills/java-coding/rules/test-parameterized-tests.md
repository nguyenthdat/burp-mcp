# test-parameterized-tests

> Use `@ParameterizedTest` for input/output variants

## Why It Matters

Copy-pasted test methods that differ only in their literal values are a maintenance trap: a bug fix in the assertion logic has to be replicated across every copy, and new edge cases require another cut-and-paste. `@ParameterizedTest` with `@CsvSource`, `@ValueSource`, or `@MethodSource` collapses these into a single, table-driven test that documents every case in one place.

## Bad

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class DiscountCalculatorTest {

    @Test
    void tenItemsGetsTenPercent() {
        assertThat(DiscountCalculator.rateFor(10)).isEqualTo(0.10);
    }

    @Test
    void twentyItemsGetsFifteenPercent() {
        assertThat(DiscountCalculator.rateFor(20)).isEqualTo(0.15);
    }

    @Test
    void fiftyItemsGetsTwentyPercent() {
        assertThat(DiscountCalculator.rateFor(50)).isEqualTo(0.20);
    }
    // three near-identical methods; adding a case means copying another one
}
```

## Good

```java
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.assertj.core.api.Assertions.assertThat;

class DiscountCalculatorTest {

    @ParameterizedTest(name = "{0} items yields a {1} discount rate")
    @CsvSource({
        "10,  0.10",
        "20,  0.15",
        "50,  0.20",
        "1,   0.00"
    })
    void ratesScaleWithQuantity(int quantity, double expectedRate) {
        assertThat(DiscountCalculator.rateFor(quantity)).isEqualTo(expectedRate);
    }
}
```

## MethodSource for Complex Arguments

```java
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.Arguments;
import org.junit.jupiter.params.provider.MethodSource;

import java.util.stream.Stream;

import static org.assertj.core.api.Assertions.assertThat;

class ShippingCostCalculatorTest {

    @ParameterizedTest
    @MethodSource("shipmentsAndExpectedCosts")
    void computesShippingCost(Shipment shipment, Money expectedCost) {
        assertThat(ShippingCostCalculator.costOf(shipment)).isEqualTo(expectedCost);
    }

    static Stream<Arguments> shipmentsAndExpectedCosts() {
        return Stream.of(
            Arguments.of(new Shipment(Region.DOMESTIC, 2.0), Money.of(4, 99)),
            Arguments.of(new Shipment(Region.INTERNATIONAL, 2.0), Money.of(24, 50)),
            Arguments.of(new Shipment(Region.DOMESTIC, 25.0), Money.of(19, 99))
        );
    }
}
```

## ValueSource for Simple Enumerations

```java
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import static org.assertj.core.api.Assertions.assertThat;

class EmailValidatorTest {

    @ParameterizedTest
    @ValueSource(strings = {"", " ", "not-an-email", "missing@", "@missing.com"})
    void rejectsInvalidAddresses(String candidate) {
        assertThat(EmailValidator.isValid(candidate)).isFalse();
    }
}
```

## See Also

- [`test-one-concept-per-test`](test-one-concept-per-test.md) - keep each parameterized case testing the same single behavior
- [`test-descriptive-names`](test-descriptive-names.md) - naming display templates for parameterized cases
- [`test-avoid-logic-in-tests`](test-avoid-logic-in-tests.md) - avoid replacing loops in tests with logic instead of parameterization
