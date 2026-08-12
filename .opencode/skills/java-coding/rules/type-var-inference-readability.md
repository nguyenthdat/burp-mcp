# type-var-inference-readability

> Use `var` only when the type is obvious from context

## Why It Matters

`var` (Java 10+) removes redundant type declarations, but readability depends entirely on the reader being able to recover the type at a glance from the right-hand side. Overusing `var` on method return values with non-obvious names forces readers to jump to the method's declaration just to know what a variable holds, which is a net readability loss disguised as a brevity win.

## Bad

```java
// Type is genuinely unclear from the call alone - what does process() return?
var result = orderService.process(order);
result.getStatus(); // reader has no idea what type result is without navigating away

// var hides a widening/narrowing surprise
var count = 100;      // int
var total = count * 1.5; // double - fine here, but easy to lose track in longer methods

// Overly clever use with a generic factory that provides no clue
var cache = Factories.create();
```

## Good

```java
// Constructor call - type is right there, var adds no ambiguity
var users = new ArrayList<User>();
var order = new Order(customerId, items);

// Obvious from a well-named factory method
var reader = Files.newBufferedReader(path);

// When the type is NOT obvious, spell it out instead of using var
ProcessingResult result = orderService.process(order);
result.getStatus();

// Explicit type also preferred when it documents an important contract
List<String> names = fetchNames(); // vs. `var names` which hides that it's a List
```

## A Practical Rule of Thumb

Use `var` when:
- The right-hand side is a constructor call (`new Foo<>()`) - the type is already spelled out.
- The right-hand side is a clearly-named factory (`Executors.newFixedThreadPool(4)`).
- The variable's scope is short (a few lines), so any ambiguity is trivially resolved by reading on.

Avoid `var` when:
- The right-hand side is a method call whose return type isn't obvious from its name.
- The variable escapes a small scope (fields, public method parameters - `var` is not legal there anyway, but the same principle applies to return types).
- The declared type documents an API contract worth keeping visible (e.g. an interface type like `List<String>` vs. a concrete `ArrayList<String>`).

## See Also

- [`modern-var-local-inference`](modern-var-local-inference.md) - The JDK feature this rule governs the style of
- [`type-generic-method-inference`](type-generic-method-inference.md) - Related inference mechanics for generic methods
- [`name-no-hungarian-notation`](name-no-hungarian-notation.md) - Naming choices matter more once the type is implicit
