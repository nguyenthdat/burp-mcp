# test-one-concept-per-test

> Assert one behavior/concept per test method

## Why It Matters

When a single test method checks five unrelated things, the first failing assertion aborts the method and hides whether the other four still pass. The test name also can't describe five behaviors at once, so it degrades into a vague `testEverything`. Splitting each concept into its own method means a failure report immediately names the exact behavior that broke.

## Bad

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class UserRegistrationTest {

    @Test
    void registrationWorks() {
        User user = UserRegistration.register("alice", "alice@example.com", "Str0ng!Pass");

        assertThat(user.username()).isEqualTo("alice"); // concept 1: username stored
        assertThat(user.isEmailVerified()).isFalse();   // concept 2: starts unverified
        assertThat(user.createdAt()).isNotNull();       // concept 3: timestamp set
        assertThat(user.roles()).containsExactly(Role.MEMBER); // concept 4: default role
        // one failure here hides whether the other three concepts still hold
    }
}
```

## Good

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class UserRegistrationTest {

    @Test
    void registeredUserRetainsProvidedUsername() {
        User user = UserRegistration.register("alice", "alice@example.com", "Str0ng!Pass");

        assertThat(user.username()).isEqualTo("alice");
    }

    @Test
    void registeredUserStartsWithUnverifiedEmail() {
        User user = UserRegistration.register("alice", "alice@example.com", "Str0ng!Pass");

        assertThat(user.isEmailVerified()).isFalse();
    }

    @Test
    void registeredUserIsAssignedDefaultMemberRole() {
        User user = UserRegistration.register("alice", "alice@example.com", "Str0ng!Pass");

        assertThat(user.roles()).containsExactly(Role.MEMBER);
    }
}
```

## Multiple Assertions on One Subject Are Fine

"One concept" doesn't mean "one `assertThat` call" — several assertions describing the same outcome on the same subject belong together, since AssertJ's soft-assertion-free chains still fail with a single clear cause.

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class MoneyTest {

    @Test
    void additionSumsAmountAndPreservesCurrency() {
        Money five = Money.of(5, "USD");
        Money ten = Money.of(10, "USD");

        Money result = five.plus(ten);

        // both assertions describe the single concept "addition produced the right Money"
        assertThat(result.amount()).isEqualTo(15);
        assertThat(result.currency()).isEqualTo("USD");
    }
}
```

## See Also

- [`test-descriptive-names`](test-descriptive-names.md) - a focused name is a symptom of a focused test
- [`test-arrange-act-assert`](test-arrange-act-assert.md) - AAA structure naturally narrows a test to one act and one assert
- [`test-avoid-logic-in-tests`](test-avoid-logic-in-tests.md) - branching logic is a common sign a test covers too much
