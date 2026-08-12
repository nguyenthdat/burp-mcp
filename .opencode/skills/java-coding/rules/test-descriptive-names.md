# test-descriptive-names

> Write descriptive test names, not `test1`/`testFoo`

## Why It Matters

A test name is the first (and often only) thing a developer reads when a CI run fails at 2am. `testFoo` or `test1` tells them nothing; they have to open the file, read the body, and reverse-engineer intent before they can even start debugging. A name like `withdrawalFailsWhenAmountExceedsBalance` turns a red test into a bug report on its own.

## Bad

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class AccountTest {

    @Test
    void test1() {
        Account account = new Account(100);
        assertThat(account.withdraw(50)).isTrue();
    }

    @Test
    void testWithdraw2() { // "2" implies a sequence, not a scenario
        Account account = new Account(100);
        assertThat(account.withdraw(150)).isFalse();
    }

    @Test
    void testFoo() { // meaningless
        Account account = new Account(0);
        assertThat(account.balance()).isZero();
    }
}
```

## Good

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class AccountTest {

    @Test
    void withdrawalSucceedsWhenAmountIsWithinBalance() {
        Account account = new Account(100);
        assertThat(account.withdraw(50)).isTrue();
    }

    @Test
    void withdrawalFailsWhenAmountExceedsBalance() {
        Account account = new Account(100);
        assertThat(account.withdraw(150)).isFalse();
    }

    @Test
    void newAccountStartsWithZeroBalance() {
        Account account = new Account(0);
        assertThat(account.balance()).isZero();
    }
}
```

## Naming Convention: `methodOrBehavior_condition_expectedResult`

```java
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class PasswordValidatorTest {

    @Test
    void validate_passwordShorterThanEightChars_throwsWeakPasswordException() {
        assertThatThrownBy(() -> PasswordValidator.validate("abc123"))
            .isInstanceOf(WeakPasswordException.class);
    }

    @Test
    void validate_passwordMeetsAllRules_returnsWithoutThrowing() {
        assertThatThrownBy(() -> PasswordValidator.validate("Str0ng!Passw0rd"))
            .doesNotThrowAnyException();
    }
}
```

Either a full-sentence style (`withdrawalFailsWhenAmountExceedsBalance`) or an underscore-segmented style (`validate_shortPassword_throws`) is acceptable — pick one convention per codebase and apply it consistently. What matters is that the name states the scenario and the expected outcome, not an index number.

## See Also

- [`test-junit5-annotations`](test-junit5-annotations.md) - using `@DisplayName` alongside descriptive method names
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - a focused name is only possible when the test checks one thing
- [`name-test-method-descriptive`](name-test-method-descriptive.md) - naming conventions for test methods generally
