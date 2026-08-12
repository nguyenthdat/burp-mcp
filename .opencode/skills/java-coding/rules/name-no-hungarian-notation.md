# name-no-hungarian-notation

> Avoid Hungarian notation and type-suffix cruft

## Why It Matters

Java's static typing and IDE tooling make type-encoding prefixes redundant: hovering over any identifier or checking the declaration tells you the type instantly, so prefixes like `strName` or `iCount` add visual noise without adding information. Worse, Hungarian-style names actively rot - a field renamed from `List` to `Set` leaves a stale `lstItems` name lying around unless every call site is also renamed, which rarely happens.

## Bad

```java
public class OrderForm {

    private String strCustomerName;   // type already obvious from declaration
    private int iQuantity;
    private List<String> lstItemIds;
    private boolean bIsValid;
    private OrderService m_orderService;  // "m_" member prefix, C++ style

    public void setStrCustomerName(String strCustomerName) {
        this.strCustomerName = strCustomerName;
    }

    public double calcTotalPriceDbl(double dblUnitPrice, int iQty) {
        return dblUnitPrice * iQty;
    }
}
```

## Good

```java
public class OrderForm {

    private String customerName;
    private int quantity;
    private List<String> itemIds;
    private boolean valid;
    private OrderService orderService;

    public void setCustomerName(String customerName) {
        this.customerName = customerName;
    }

    public double calculateTotalPrice(double unitPrice, int quantity) {
        return unitPrice * quantity;
    }
}
```

## Interface Implementation Suffixes

The one narrow exception the Java ecosystem tolerates is suffixing an implementation class with `Impl` when a single canonical implementation exists for an interface and no more descriptive qualifier applies - but prefer a descriptive name (`InMemoryUserRepository` over `UserRepositoryImpl`) whenever more than one implementation is plausible.

```java
public interface UserRepository {
    Optional<User> findById(String id);
}

// Acceptable when there is exactly one implementation and no better name exists
class UserRepositoryImpl implements UserRepository {
    // ...
}

// Preferred when the implementation detail is meaningful
class JdbcUserRepository implements UserRepository {
    // ...
}

class InMemoryUserRepository implements UserRepository {
    // ...
}
```

## See Also

- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
- [`name-acronyms-as-words`](name-acronyms-as-words.md) - Treat acronyms as words in identifiers
- [`api-minimal-public-surface`](api-minimal-public-surface.md) - Keep the public API surface minimal
