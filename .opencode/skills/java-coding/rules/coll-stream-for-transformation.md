# coll-stream-for-transformation

> Use the Stream API for transformation pipelines

## Why It Matters

When a task is genuinely "take a collection, filter/map/reduce it, produce a result," the Stream API expresses that pipeline declaratively in one readable chain instead of scattering intermediate mutable variables across a loop body. It also composes cleanly - each stage can be reasoned about independently - and avoids the class of bugs where a loop variable is reused or a condition is applied in the wrong order.

## Bad

```java
// Manual loop with intermediate mutable collections at every stage
List<Order> orders = fetchOrders();
List<Order> shipped = new ArrayList<>();
for (Order order : orders) {
    if (order.status() == OrderStatus.SHIPPED) {
        shipped.add(order);
    }
}

List<BigDecimal> totals = new ArrayList<>();
for (Order order : shipped) {
    totals.add(order.total());
}

BigDecimal sum = BigDecimal.ZERO;
for (BigDecimal total : totals) {
    sum = sum.add(total);
}
```

## Good

```java
// Single declarative pipeline - filter, map, reduce
List<Order> orders = fetchOrders();
BigDecimal sum = orders.stream()
    .filter(order -> order.status() == OrderStatus.SHIPPED)
    .map(Order::total)
    .reduce(BigDecimal.ZERO, BigDecimal::add);
```

## Transforming Into a New Shape

```java
List<Order> orders = fetchOrders();

// Extract a projection into a new List
List<String> customerEmails = orders.stream()
    .map(Order::customer)
    .map(Customer::email)
    .distinct()
    .toList();

// Filter and collect into a Map keyed by order id
Map<Long, Order> shippedById = orders.stream()
    .filter(order -> order.status() == OrderStatus.SHIPPED)
    .collect(Collectors.toMap(Order::id, Function.identity()));
```

## When a Loop Is Still Better

If the body needs multiple exit points, complex branching, or checked exceptions per element, a plain `for` loop is often clearer than contorting a stream around it. See `coll-stream-vs-loop` for that trade-off.

## See Also

- [`coll-stream-vs-loop`](coll-stream-vs-loop.md) - Deciding when a loop communicates intent better than a stream
- [`coll-collectors-toX`](coll-collectors-toX.md) - Choosing the right terminal collector
- [`coll-avoid-side-effects-streams`](coll-avoid-side-effects-streams.md) - Keeping stream pipelines free of mutation
- [`modern-var-local-inference`](modern-var-local-inference.md) - Readable local variable typing inside pipelines
