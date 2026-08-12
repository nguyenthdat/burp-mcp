# test-arrange-act-assert

> Structure tests as arrange/act/assert

## Why It Matters

The AAA pattern gives every test a predictable shape: set up state, perform the one action under test, then verify the outcome. Tests that interleave setup, action, and assertion force readers to trace variable mutations line by line to figure out what's actually being verified, and they make it easy to accidentally assert on stale state.

## Bad

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class ShoppingCartTest {

    @Test
    void cartTotalsAndAppliesDiscount() {
        ShoppingCart cart = new ShoppingCart();
        cart.add(new Item("Widget", 25.00));
        assertThat(cart.itemCount()).isEqualTo(1); // asserting mid-setup
        cart.add(new Item("Gadget", 15.00));
        cart.applyCoupon("SAVE10");
        assertThat(cart.total()).isEqualTo(36.00); // arrange, act, and assert all tangled together
        cart.add(new Item("Gizmo", 5.00)); // mutating state after already asserting
        assertThat(cart.total()).isEqualTo(40.50);
    }
}
```

## Good

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class ShoppingCartTest {

    @Test
    void couponAppliesTenPercentDiscountToSubtotal() {
        // Arrange
        ShoppingCart cart = new ShoppingCart();
        cart.add(new Item("Widget", 25.00));
        cart.add(new Item("Gadget", 15.00));

        // Act
        cart.applyCoupon("SAVE10");

        // Assert
        assertThat(cart.total()).isEqualTo(36.00);
    }
}
```

## Complex Arrange with a Builder

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class LoanEligibilityTest {

    @Test
    void applicantWithLowDebtToIncomeRatioIsApproved() {
        // Arrange
        Applicant applicant = Applicant.builder()
            .annualIncome(90_000)
            .existingMonthlyDebt(500)
            .creditScore(720)
            .build();
        LoanRequest request = new LoanRequest(applicant, /* amount */ 20_000);

        // Act
        EligibilityResult result = LoanEligibilityEngine.evaluate(request);

        // Assert
        assertThat(result.isApproved()).isTrue();
        assertThat(result.reason()).isEmpty();
    }
}
```

## Given/When/Then as an Equivalent Convention

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class ParkingGarageTest {

    @Test
    void gateRejectsEntryWhenGarageIsFull() {
        // Given
        ParkingGarage garage = ParkingGarage.withCapacity(1);
        garage.admit(new Vehicle("ABC-123"));

        // When
        EntryResult result = garage.admit(new Vehicle("XYZ-999"));

        // Then
        assertThat(result).isEqualTo(EntryResult.REJECTED_FULL);
    }
}
```

## See Also

- [`test-descriptive-names`](test-descriptive-names.md) - naming the single behavior the Act step exercises
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - keeping the Assert step focused on one outcome
- [`test-avoid-logic-in-tests`](test-avoid-logic-in-tests.md) - keeping Arrange/Act/Assert free of conditionals
