# name-test-method-descriptive

> Name test methods descriptively, not `test1`

## Why It Matters

A test name is the first (and sometimes only) piece of information a developer sees when a CI run fails - `test1` or `testOrder` tells them nothing, forcing them to open the test body just to understand what broke. Descriptive names that state the scenario and expected outcome turn a red build into self-documenting output and double as executable specification of the code's behavior.

## Bad

```java
class OrderServiceTest {

    @Test
    void test1() {
        Order order = new Order(100.0);
        assertTrue(order.isValid());
    }

    @Test
    void testOrder() {  // which behavior of "order" is being tested?
        Order order = new Order(-50.0);
        assertFalse(order.isValid());
    }

    @Test
    void testDiscount() {  // no indication of the condition or expectation
        Order order = new Order(200.0);
        order.applyDiscount(0.1);
        assertEquals(180.0, order.getTotal());
    }
}
```

## Good

```java
class OrderServiceTest {

    @Test
    void isValid_returnsTrue_whenTotalIsPositive() {
        Order order = new Order(100.0);
        assertTrue(order.isValid());
    }

    @Test
    void isValid_returnsFalse_whenTotalIsNegative() {
        Order order = new Order(-50.0);
        assertFalse(order.isValid());
    }

    @Test
    void applyDiscount_reducesTotal_byGivenPercentage() {
        Order order = new Order(200.0);
        order.applyDiscount(0.1);
        assertEquals(180.0, order.getTotal());
    }
}
```

## Alternative Styles

JUnit 5's `@DisplayName` lets a test carry a readable sentence separately from a method name that still follows Java identifier rules, which is useful when the desired description reads more naturally in prose than in `camelCase`:

```java
class OrderServiceTest {

    @Test
    @DisplayName("applying a 10% discount reduces the order total accordingly")
    void applyDiscount_reducesTotal_byGivenPercentage() {
        Order order = new Order(200.0);
        order.applyDiscount(0.1);
        assertEquals(180.0, order.getTotal());
    }
}
```

Whichever convention a codebase adopts (`given_when_then`, `methodName_condition_expectedResult`, or plain prose sentences), the important part is consistency: every test name should let a reader guess the assertion before opening the method body.

## See Also

- [`test-descriptive-names`](test-descriptive-names.md) - Write descriptive test names throughout the suite
- [`test-nested-grouping`](test-nested-grouping.md) - Group related tests with @Nested classes
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - Test one concept per test method
- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
