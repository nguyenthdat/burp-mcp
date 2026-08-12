# test-integration-test-separation

> Separate unit tests from integration tests

## Why It Matters

Unit tests and integration tests have different jobs and different cost profiles: unit tests should run in milliseconds with no I/O and give tight feedback on logic, while integration tests exercise real databases, HTTP clients, or message brokers and are slower and more brittle. Mixing them in the same source set means a single flaky container spins up on every `mvn test`, slowing down the fast feedback loop that makes unit tests valuable in the first place.

## Bad

```java
import org.junit.jupiter.api.Test;
import org.testcontainers.containers.PostgreSQLContainer;

import static org.assertj.core.api.Assertions.assertThat;

// lives in src/test/java alongside pure unit tests, runs on every `mvn test`
class OrderRepositoryTest {

    private static final PostgreSQLContainer<?> POSTGRES =
        new PostgreSQLContainer<>("postgres:16").withReuse(true);

    @Test
    void savesAndReloadsOrder() {
        POSTGRES.start(); // spins up a real container in what looks like a plain unit test
        OrderRepository repo = new JdbcOrderRepository(POSTGRES.getJdbcUrl());

        repo.save(new Order("ord-1", 42.0));
        Order reloaded = repo.findById("ord-1");

        assertThat(reloaded.total()).isEqualTo(42.0);
    }
}
// a developer running `mvn test -Dtest=OrderCalculatorTest` for quick feedback
// unknowingly still triggers Docker if the build wires all tests into one phase
```

## Good

```java
// src/test/java/.../OrderCalculatorTest.java  (unit test, no I/O, runs in the default `test` phase)
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class OrderCalculatorTest {

    @Test
    void appliesTaxToLineItemTotal() {
        OrderCalculator calculator = new OrderCalculator(new TaxCalculator());

        assertThat(calculator.totalFor(lineItems(), "US-CA")).isEqualTo(107.25);
    }
}
```

```java
// src/it/java/.../OrderRepositoryIT.java (integration test, isolated naming + build phase)
import org.junit.jupiter.api.Test;
import org.testcontainers.containers.PostgreSQLContainer;
import org.testcontainers.junit.jupiter.Container;
import org.testcontainers.junit.jupiter.Testcontainers;

import static org.assertj.core.api.Assertions.assertThat;

@Testcontainers
class OrderRepositoryIT {

    @Container
    private static final PostgreSQLContainer<?> POSTGRES =
        new PostgreSQLContainer<>("postgres:16");

    @Test
    void savesAndReloadsOrder() {
        OrderRepository repo = new JdbcOrderRepository(POSTGRES.getJdbcUrl());

        repo.save(new Order("ord-1", 42.0));
        Order reloaded = repo.findById("ord-1");

        assertThat(reloaded.total()).isEqualTo(42.0);
    }
}
```

## Wiring the Build to Separate Phases

```xml
<!-- pom.xml: Failsafe runs *IT classes in the integration-test phase, separate from Surefire's *Test in `test` -->
<build>
  <plugins>
    <plugin>
      <groupId>org.apache.maven.plugins</groupId>
      <artifactId>maven-failsafe-plugin</artifactId>
      <executions>
        <execution>
          <goals>
            <goal>integration-test</goal>
            <goal>verify</goal>
          </goals>
        </execution>
      </executions>
    </plugin>
  </plugins>
</build>
```

With this split, `mvn test` gives fast unit-only feedback, and `mvn verify` additionally runs the Testcontainers-backed integration suite in CI.

## See Also

- [`test-mock-boundaries-not-internals`](test-mock-boundaries-not-internals.md) - what unit tests should fake so integration tests can verify the real thing
- [`test-testinstance-lifecycle`](test-testinstance-lifecycle.md) - sharing expensive fixtures like containers within a class
- [`proj-package-by-feature`](proj-package-by-feature.md) - structuring source sets so test types stay physically separated
