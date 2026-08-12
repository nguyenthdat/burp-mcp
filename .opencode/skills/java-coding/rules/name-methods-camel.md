# name-methods-camel

> Use `camelCase` for methods and fields

## Why It Matters

`camelCase` for methods and fields is the single most consistent convention in the Java ecosystem, and every framework, IDE autocomplete, and code generator (Lombok, JAXB, Jackson) relies on it to distinguish members from types and constants at a glance. Inconsistent casing forces reviewers to double-check whether an identifier is a field, a method, or a class every time it appears.

## Bad

```java
public class OrderService {

    private final OrderRepository order_repository;  // snake_case field
    private int RetryCount;                            // PascalCase field, reads like a type

    public OrderService(OrderRepository order_repository) {
        this.order_repository = order_repository;
    }

    public Order Process_Order(String order_id) {  // mixed-case, underscores
        return order_repository.FindById(order_id);  // inconsistent casing on calls
    }

    public boolean IsValid(Order o) {  // PascalCase method name
        return o != null;
    }
}
```

## Good

```java
public class OrderService {

    private final OrderRepository orderRepository;
    private int retryCount;

    public OrderService(OrderRepository orderRepository) {
        this.orderRepository = orderRepository;
    }

    public Order processOrder(String orderId) {
        return orderRepository.findById(orderId);
    }

    public boolean isValid(Order order) {
        return order != null;
    }
}
```

## Acronyms and Abbreviations

Acronyms inside camelCase names follow the same rule as class names: only the first letter is capitalized when the acronym is not leading, and it is lowercase entirely when it starts the identifier.

```java
private String userId;      // not userID
private URL homePageUrl;    // not homePageURL
public void parseXmlPayload(String xml) { }  // not parseXMLPayload
```

## See Also

- [`name-classes-pascal`](name-classes-pascal.md) - Use PascalCase for classes, interfaces, enums, records
- [`name-constants-screaming-snake`](name-constants-screaming-snake.md) - Use SCREAMING_SNAKE_CASE for constants
- [`name-acronyms-as-words`](name-acronyms-as-words.md) - Treat acronyms as words in identifiers
- [`name-getter-setter-bean-convention`](name-getter-setter-bean-convention.md) - Follow JavaBean naming for accessors
