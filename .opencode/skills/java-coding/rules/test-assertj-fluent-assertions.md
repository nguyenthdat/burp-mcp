# test-assertj-fluent-assertions

> Use AssertJ fluent assertions over raw JUnit asserts

## Why It Matters

JUnit's `assertEquals`/`assertTrue` produce terse failure messages ("expected: <true> but was: <false>") that force developers to re-read the assertion to understand what was actually being checked. AssertJ's fluent API reads like a sentence, chains multiple checks on the same value, and produces rich, targeted failure diagnostics — especially for collections, exceptions, and object graphs.

## Bad

```java
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class InvoiceTest {

    @Test
    void invoiceTotalsAreCorrect() {
        Invoice invoice = Invoice.of(lineItems());

        assertTrue(invoice.total() > 0); // failure gives no context on the actual value
        assertEquals(3, invoice.lineItems().size());
        assertTrue(invoice.lineItems().stream().anyMatch(li -> li.name().equals("Widget")));
        assertNotNull(invoice.dueDate());
        assertFalse(invoice.isPaid());
    }
}
```

## Good

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class InvoiceTest {

    @Test
    void invoiceTotalsAreCorrect() {
        Invoice invoice = Invoice.of(lineItems());

        assertThat(invoice.total()).isPositive();
        assertThat(invoice.lineItems())
            .hasSize(3)
            .extracting(LineItem::name)
            .contains("Widget");
        assertThat(invoice.dueDate()).isNotNull();
        assertThat(invoice.isPaid()).isFalse();
    }
}
```

## Exception Assertions

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class WithdrawalTest {

    @Test
    void withdrawalBeyondBalanceIsRejected() {
        Account account = Account.withBalance(50);

        assertThatThrownBy(() -> account.withdraw(100))
            .isInstanceOf(InsufficientFundsException.class)
            .hasMessageContaining("insufficient funds")
            .hasMessageContaining("50");

        assertThat(account.balance()).isEqualTo(50);
    }
}
```

## Object and Collection Assertions

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class UserRepositoryTest {

    @Test
    void findsActiveUsersOrderedByName() {
        UserRepository repo = new InMemoryUserRepository(sampleUsers());

        java.util.List<User> active = repo.findActive();

        assertThat(active)
            .extracting(User::name)
            .containsExactly("Ana", "Bob", "Cleo");

        assertThat(active.get(0))
            .usingRecursiveComparison()
            .ignoringFields("lastLoginAt")
            .isEqualTo(new User("Ana", "ana@example.com", true, null));
    }
}
```

## See Also

- [`test-mockito-mocking`](test-mockito-mocking.md) - combining stubbed collaborators with AssertJ assertions
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - chaining assertions on one subject, not many subjects
- [`err-exception-message-context`](err-exception-message-context.md) - why exception messages need to carry context AssertJ can assert on
