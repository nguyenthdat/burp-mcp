# test-mock-boundaries-not-internals

> Mock at architectural boundaries, not internal collaborators

## Why It Matters

Mocking every internal helper class couples the test to the implementation's current internal structure — refactor how `OrderService` computes a total internally, and tests break even though behavior is unchanged. Mocking should be reserved for true architectural boundaries (network calls, databases, clocks, external systems); internal collaborators should run for real so tests verify actual behavior, not a rehearsed script of internal calls.

## Bad

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OrderServiceTest {

    @Mock
    private TaxCalculator taxCalculator; // pure internal helper, no I/O, no boundary

    @Mock
    private DiscountCalculator discountCalculator; // same - internal, deterministic logic

    @Test
    void placesOrderWithCorrectTotal() {
        when(taxCalculator.taxFor(100.0)).thenReturn(8.0);
        when(discountCalculator.discountFor(100.0)).thenReturn(10.0);

        OrderService service = new OrderService(taxCalculator, discountCalculator, mockPaymentGateway());
        Order order = service.placeOrder(lineItems(), "US-CA");

        assertThat(order.total()).isEqualTo(98.0);
        // this only proves the mocks return what we told them to - real tax/discount
        // logic is never exercised, and refactoring the calculators breaks this test
        // for reasons unrelated to OrderService's actual behavior
    }
}
```

## Good

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OrderServiceTest {

    @Mock
    private PaymentGateway paymentGateway; // real boundary: an external payment provider

    @Test
    void placesOrderWithCorrectTotal() {
        // real TaxCalculator and DiscountCalculator - fast, deterministic, no I/O
        OrderService service = new OrderService(
            new TaxCalculator(), new DiscountCalculator(), paymentGateway);
        when(paymentGateway.charge(anyDouble())).thenReturn(PaymentResult.success());

        Order order = service.placeOrder(lineItems(), "US-CA");

        assertThat(order.total()).isEqualTo(98.0); // exercises real tax/discount logic
        verify(paymentGateway).charge(98.0); // only the true external boundary is mocked
    }
}
```

## Boundaries Worth Mocking vs. Internals Worth Running for Real

```java
// Good candidates for @Mock: network clients, database repositories, message
// publishers, clocks, random number generators, file systems, third-party SDKs.
//
// Poor candidates for @Mock: value objects, pure calculators, validators,
// mappers, anything with no side effects and no I/O - let them run.

class PricingEngineTest {
    // PricingEngine depends on TaxCalculator (pure) and InventoryClient (network boundary)

    @org.mockito.Mock
    private InventoryClient inventoryClient; // mocked: real boundary

    @Test
    void computesPriceIncludingLiveStockSurcharge() {
        when(inventoryClient.stockLevel("SKU-1")).thenReturn(2);

        PricingEngine engine = new PricingEngine(new TaxCalculator(), inventoryClient);

        assertThat(engine.priceFor("SKU-1", 100.0)).isEqualTo(115.0); // low-stock surcharge applied
    }
}
```

## See Also

- [`test-mockito-mocking`](test-mockito-mocking.md) - mechanics of using Mockito once boundaries are identified
- [`test-integration-test-separation`](test-integration-test-separation.md) - verifying real boundary implementations separately
- [`api-composition-over-inheritance`](api-composition-over-inheritance.md) - designing collaborators so boundaries are easy to identify and inject
