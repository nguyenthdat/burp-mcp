# err-unchecked-wrap-checked

> Wrap checked exceptions instead of propagating `throws` everywhere

## Why It Matters

When a checked exception from a low-level dependency (JDBC's `SQLException`, `IOException`) is declared with `throws` on every method up the call stack, every intermediate layer that has no meaningful way to recover from it is forced to either declare it too or immediately wrap it — polluting unrelated interfaces with implementation details of a library three layers down. Wrapping once, near the source, in an unchecked exception with a clear domain meaning keeps the `throws` clause of your public API meaningful and stable.

## Bad

```java
// Every layer up the stack must repeat this checked exception in its signature,
// even layers that have no idea what a SQLException even means for them
public interface OrderRepository {
    Order findById(String id) throws SQLException;
}

public class OrderService {
    public Order getOrder(String id) throws SQLException { // leaking a JDBC detail
        return repository.findById(id);
    }
}

public class OrderController {
    public Response handleGetOrder(String id) throws SQLException { // now the HTTP layer knows about JDBC
        return Response.ok(orderService.getOrder(id));
    }
}
```

## Good

```java
public class OrderPersistenceException extends RuntimeException {
    public OrderPersistenceException(String message, Throwable cause) {
        super(message, cause);
    }
}

public interface OrderRepository {
    Order findById(String id); // no leaked implementation detail
}

public class JdbcOrderRepository implements OrderRepository {
    @Override
    public Order findById(String id) {
        try {
            return jdbcTemplate.queryForObject(SELECT_BY_ID, this::mapRow, id);
        } catch (SQLException e) {
            // Wrapped once, right at the boundary where the checked exception originates
            throw new OrderPersistenceException("failed to load order " + id, e);
        }
    }
}

// Every layer above this one has a clean, stable signature
public class OrderService {
    public Order getOrder(String id) {
        return repository.findById(id); // OrderPersistenceException propagates unchecked if it happens
    }
}
```

## See Also

- [`err-checked-vs-unchecked`](err-checked-vs-unchecked.md) - Choose checked vs unchecked by recoverability
- [`err-exception-chaining`](err-exception-chaining.md) - Chain causes via constructor
- [`anti-excessive-checked-exceptions`](anti-excessive-checked-exceptions.md) - Avoid excessive checked exceptions
- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - Build a custom exception hierarchy for domain errors
