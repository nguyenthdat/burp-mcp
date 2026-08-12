# test-junit5-extensions

> Use JUnit 5 extensions for cross-cutting test setup

## Why It Matters

Copy-pasting the same `@BeforeEach`/`@AfterEach` setup (spinning up a test double server, starting a container, seeding a clock) into every test class duplicates lifecycle code and makes it easy for one copy to drift out of sync. JUnit 5's `Extension` API lets that lifecycle live in one reusable class, applied declaratively with `@ExtendWith`, so test classes stay focused on behavior instead of plumbing.

## Bad

```java
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

class OrderServiceTest {

    private WireMockServer wireMock;

    @BeforeEach
    void startWireMock() { // duplicated verbatim in every test class that needs a fake HTTP server
        wireMock = new WireMockServer(0);
        wireMock.start();
        System.setProperty("payments.api.url", wireMock.baseUrl());
    }

    @AfterEach
    void stopWireMock() {
        wireMock.stop();
    }

    @Test
    void placesOrderSuccessfully() {
        // test body
    }
}
```

## Good

```java
import org.junit.jupiter.api.extension.AfterEachCallback;
import org.junit.jupiter.api.extension.BeforeEachCallback;
import org.junit.jupiter.api.extension.ExtensionContext;

public class WireMockExtension implements BeforeEachCallback, AfterEachCallback {

    private WireMockServer wireMock;

    @Override
    public void beforeEach(ExtensionContext context) {
        wireMock = new WireMockServer(0);
        wireMock.start();
        System.setProperty("payments.api.url", wireMock.baseUrl());
    }

    @Override
    public void afterEach(ExtensionContext context) {
        wireMock.stop();
    }

    public WireMockServer server() {
        return wireMock;
    }
}
```

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;

@ExtendWith(WireMockExtension.class)
class OrderServiceTest {

    @Test
    void placesOrderSuccessfully() {
        // extension already started the fake server before this method runs
    }
}
```

## Built-in Extensions Worth Knowing

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension; // wires @Mock fields, validates stubs

@ExtendWith(MockitoExtension.class)
class PaymentProcessorTest {

    @org.mockito.Mock
    private PaymentGateway gateway;

    @Test
    void processesPaymentThroughGateway() {
        // gateway is already a Mockito mock, injected by the extension
    }
}
```

`@ExtendWith` also composes with Spring's `SpringExtension`, Testcontainers' container lifecycle extensions, and custom parameter resolvers (`ParameterResolver`) for injecting fixtures directly as test method arguments.

## Parameter Resolver Example

```java
import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.jupiter.api.extension.ParameterContext;
import org.junit.jupiter.api.extension.ParameterResolver;

public class ClockExtension implements ParameterResolver {

    @Override
    public boolean supportsParameter(ParameterContext pc, ExtensionContext ec) {
        return pc.getParameter().getType() == java.time.Clock.class;
    }

    @Override
    public Object resolveParameter(ParameterContext pc, ExtensionContext ec) {
        return java.time.Clock.fixed(
            java.time.Instant.parse("2026-01-01T00:00:00Z"),
            java.time.ZoneOffset.UTC);
    }
}
```

## See Also

- [`test-junit5-annotations`](test-junit5-annotations.md) - the annotation model extensions plug into
- [`test-mockito-mocking`](test-mockito-mocking.md) - `MockitoExtension` as a concrete example
- [`test-integration-test-separation`](test-integration-test-separation.md) - extensions commonly used to manage containers in integration tests
