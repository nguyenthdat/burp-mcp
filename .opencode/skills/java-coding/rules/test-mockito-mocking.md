# test-mockito-mocking

> Use Mockito for mocking dependencies

## Why It Matters

Hand-rolled stub classes and manual test doubles duplicate the interface they fake, drift out of sync as the interface evolves, and hide intent behind boilerplate. Mockito lets a test declare exactly the interactions it cares about — stubbed return values and verified calls — while `@ExtendWith(MockitoExtension.class)` wires mocks up automatically and catches unused stubs.

## Bad

```java
class NotificationServiceTest {

    // hand-written fake duplicating the whole interface, most methods unused
    static class FakeEmailGateway implements EmailGateway {
        boolean sendCalled = false;

        @Override
        public void send(String to, String subject, String body) {
            sendCalled = true;
        }

        @Override
        public void sendBulk(java.util.List<String> recipients, String subject, String body) {
            throw new UnsupportedOperationException(); // dead code nobody maintains
        }
    }

    @Test
    void sendsWelcomeEmail() {
        FakeEmailGateway gateway = new FakeEmailGateway();
        NotificationService service = new NotificationService(gateway);

        service.sendWelcomeEmail("new-user@example.com");

        assert gateway.sendCalled; // no message on failure, easy to miss what was checked
    }
}
```

## Good

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class NotificationServiceTest {

    @Mock
    private EmailGateway emailGateway;

    @Test
    void sendsWelcomeEmail() {
        NotificationService service = new NotificationService(emailGateway);

        service.sendWelcomeEmail("new-user@example.com");

        verify(emailGateway).send(
            eq("new-user@example.com"),
            eq("Welcome!"),
            eq("Thanks for signing up."));
    }
}
```

## Stubbing Return Values

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class PricingServiceTest {

    @Mock
    private TaxRateProvider taxRateProvider;

    @Test
    void appliesRegionalTaxRate() {
        when(taxRateProvider.rateFor("CA")).thenReturn(0.0725);

        PricingService pricing = new PricingService(taxRateProvider);
        Money total = pricing.priceWithTax(Money.of(100, 0), "CA");

        assertThat(total).isEqualTo(Money.of(107, 25));
    }
}
```

## Argument Captors for Complex Verification

```java
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class AuditLoggerTest {

    @Mock
    private AuditSink sink;

    @Captor
    private ArgumentCaptor<AuditEvent> eventCaptor;

    @Test
    void logsUserDeactivationWithReason() {
        AuditLogger logger = new AuditLogger(sink);

        logger.recordDeactivation("user-42", "policy violation");

        verify(sink).record(eventCaptor.capture());
        AuditEvent event = eventCaptor.getValue();
        assertThat(event.subjectId()).isEqualTo("user-42");
        assertThat(event.reason()).isEqualTo("policy violation");
    }
}
```

## See Also

- [`test-mock-boundaries-not-internals`](test-mock-boundaries-not-internals.md) - what to mock and what to leave real
- [`test-assertj-fluent-assertions`](test-assertj-fluent-assertions.md) - pairing Mockito verification with AssertJ assertions
- [`test-junit5-extensions`](test-junit5-extensions.md) - how `MockitoExtension` plugs into JUnit 5
