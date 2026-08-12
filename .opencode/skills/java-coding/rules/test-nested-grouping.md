# test-nested-grouping

> Group related tests with `@Nested`

## Why It Matters

A flat test class with fifty methods forces readers to scan names for a shared prefix (`whenAccountIsFrozen_*`) to find related scenarios. `@Nested` classes turn those prefixes into an actual hierarchy: each inner class gets its own `@BeforeEach`, its own `@DisplayName`, and the generated test report reads as nested sections instead of a flat, alphabetized list.

## Bad

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class AccountTest {

    @Test
    void whenAccountIsActive_withdrawSucceeds() { /* ... */ }

    @Test
    void whenAccountIsActive_depositSucceeds() { /* ... */ }

    @Test
    void whenAccountIsFrozen_withdrawFails() { /* ... */ }

    @Test
    void whenAccountIsFrozen_depositFails() { /* ... */ }

    @Test
    void whenAccountIsClosed_withdrawFails() { /* ... */ }

    @Test
    void whenAccountIsClosed_depositFails() { /* ... */ }
    // shared setup per "when" group is duplicated in every method
}
```

## Good

```java
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class AccountTest {

    private Account account;

    @Nested
    @DisplayName("when the account is active")
    class WhenActive {

        @BeforeEach
        void openActiveAccount() {
            account = Account.open(100);
        }

        @Test
        @DisplayName("withdraw succeeds within balance")
        void withdrawSucceeds() {
            assertThat(account.withdraw(50)).isTrue();
        }

        @Test
        @DisplayName("deposit increases balance")
        void depositSucceeds() {
            account.deposit(25);
            assertThat(account.balance()).isEqualTo(125);
        }
    }

    @Nested
    @DisplayName("when the account is frozen")
    class WhenFrozen {

        @BeforeEach
        void openAndFreezeAccount() {
            account = Account.open(100);
            account.freeze();
        }

        @Test
        @DisplayName("withdraw is rejected")
        void withdrawFails() {
            assertThat(account.withdraw(50)).isFalse();
        }

        @Test
        @DisplayName("deposit is rejected")
        void depositFails() {
            assertThat(account.deposit(25)).isFalse();
        }
    }
}
```

## Deeper Nesting for Multi-Dimensional Scenarios

```java
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class DiscountEngineTest {

    @Nested
    class ForPremiumCustomers {

        @Nested
        class DuringHolidaySale {

            @Test
            void stacksLoyaltyAndSeasonalDiscount() {
                assertThat(DiscountEngine.rateFor(CustomerTier.PREMIUM, Season.HOLIDAY))
                    .isEqualTo(0.30);
            }
        }
    }
}
```

Note: inner classes annotated with `@Nested` must not be `static` — JUnit 5 needs an instance tied to the outer test class to share fields like `account` across nested tests.

## See Also

- [`test-junit5-annotations`](test-junit5-annotations.md) - lifecycle annotations that `@Nested` classes inherit and override
- [`test-testinstance-lifecycle`](test-testinstance-lifecycle.md) - how instance lifecycle interacts with nested classes
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - keeping each nested test focused on a single behavior
