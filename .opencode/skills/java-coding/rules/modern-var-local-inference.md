# modern-var-local-inference

> Use `var` for local variable type inference where it aids readability

## Why It Matters

Repeating a generic type on both sides of a declaration (`Map<String, List<Order>> map = new HashMap<String, List<Order>>();`) adds visual weight without adding information the reader didn't already have from the constructor call. `var` (Java 10+) lets the compiler infer the local variable's static type from the initializer, shortening declarations to their essential content while remaining fully statically typed - `var` is not dynamic typing, the type is fixed at compile time.

## Bad

```java
Map<String, List<Order>> ordersByCustomer = new HashMap<String, List<Order>>();

BufferedReader reader = new BufferedReader(new FileReader("data.txt"));

for (Map.Entry<String, List<Order>> entry : ordersByCustomer.entrySet()) {
    String customer = entry.getKey();
    List<Order> orders = entry.getValue();
    System.out.println(customer + ": " + orders.size());
}
```

## Good

```java
var ordersByCustomer = new HashMap<String, List<Order>>();

var reader = new BufferedReader(new FileReader("data.txt"));

for (var entry : ordersByCustomer.entrySet()) {
    var customer = entry.getKey();
    var orders = entry.getValue();
    System.out.println(customer + ": " + orders.size());
}
```

## `var` Is Still Statically Typed

```java
var count = 10;       // inferred as int, forever - not reassignable to a String
// count = "ten";     // compile error: incompatible types

var list = new ArrayList<String>();
// list.add(42);      // compile error: List<String> doesn't accept Integer
```

`var` cannot be used for fields, method parameters, or return types - only local variables, `for` loop indices, and try-with-resources resources. It also cannot infer from `null` or from an ambiguous target like a lambda expression without an explicit functional interface type.

## See Also

- [`type-var-inference-readability`](type-var-inference-readability.md) - The judgment call on when `var` helps vs. hurts readability
- [`type-generic-method-inference`](type-generic-method-inference.md) - Related inference mechanics for method calls
- [`modern-text-blocks`](modern-text-blocks.md) - Another Java 10+/15+ conciseness feature often paired with `var`
