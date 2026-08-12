# test-junit5-annotations

> Use JUnit 5 (`@Test`, `@Nested`, `@DisplayName`) idiomatically

## Why It Matters

JUnit 5's annotation model replaces JUnit 4's rigid conventions with composable, expressive building blocks. Using `@DisplayName`, `@Nested`, and lifecycle annotations correctly produces test output that reads like documentation and makes failures easier to triage in CI logs. Mixing JUnit 4 idioms (`@Before`, `@RunWith`) with JUnit 5 leads to silently skipped tests and confusing build failures.

## Bad

```java
import org.junit.Before; // JUnit 4 import mixed into a JUnit 5 test class
import org.junit.Test;

public class AccountServiceTest {

    private AccountService service;

    @Before // wrong package - this annotation is a no-op under the JUnit 5 engine
    public void init() {
        service = new AccountService(new InMemoryAccountRepository());
    }

    @Test
    public void test1() { // no @DisplayName, cryptic name, no lifecycle guarantees
        service.open("acc-1", 100);
        Assert.assertEquals(100, service.balanceOf("acc-1")); // JUnit 4 Assert API
    }
}
```

## Good

```java
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class AccountServiceTest {

    private AccountService service;

    @BeforeEach
    void setUp() {
        service = new AccountService(new InMemoryAccountRepository());
    }

    @Test
    @DisplayName("opening an account with an initial deposit sets the starting balance")
    void openAccountSetsInitialBalance() {
        service.open("acc-1", 100);

        assertThat(service.balanceOf("acc-1")).isEqualTo(100);
    }
}
```

## Lifecycle Annotations Reference

```java
import org.junit.jupiter.api.*;

class OrderProcessorTest {

    @BeforeAll
    static void startEmbeddedBroker() { /* runs once, must be static by default */ }

    @BeforeEach
    void openConnection() { /* runs before every test method */ }

    @AfterEach
    void closeConnection() { /* runs after every test method */ }

    @AfterAll
    static void stopEmbeddedBroker() { /* runs once after all tests */ }

    @Test
    @Disabled("blocked on ORD-482 until refund API ships")
    void refundIsIssuedOnCancellation() { /* skipped with a documented reason */ }
}
```

## See Also

- [`test-nested-grouping`](test-nested-grouping.md) - organizing related tests with `@Nested`
- [`test-testinstance-lifecycle`](test-testinstance-lifecycle.md) - choosing per-method vs per-class instance lifecycle
- [`test-descriptive-names`](test-descriptive-names.md) - naming tests so `@DisplayName` isn't doing all the work
- [`test-junit5-extensions`](test-junit5-extensions.md) - cross-cutting setup beyond `@BeforeEach`/`@AfterEach`
