# type-generic-method-inference

> Rely on generic method type inference over explicit witnesses

## Why It Matters

Explicit type witnesses (`Collections.<String>emptyList()`) add visual noise that the compiler can almost always figure out on its own from the target context. Since Java 8's improved target-type inference, forcing explicit witnesses is a sign the code is fighting the type system rather than working with it, and it often masks a design that would be clearer with a differently-typed variable.

## Bad

```java
// Explicit witness required because there's no target type to infer from
List<String> list = Collections.<String>emptyList();

// Forcing a witness because the assignment is split across statements
var result = Stream.<Integer>of(1, 2, 3).collect(Collectors.toList());

// Verbose explicit witness on a generic factory method
Map<String, List<Integer>> map = Utils.<String, List<Integer>>newMap();

public static <T> List<T> wrap(T item) {
    return List.<T>of(item); // redundant witness inside the method itself
}
```

## Good

```java
// Target type (the declared variable type) drives inference automatically
List<String> list = Collections.emptyList();

List<Integer> result = Stream.of(1, 2, 3).collect(Collectors.toList());

Map<String, List<Integer>> map = Utils.newMap();

public static <T> List<T> wrap(T item) {
    return List.of(item); // T inferred from the item parameter
}
```

## When a Witness Is Actually Needed

Explicit witnesses are unavoidable when there is no target type to infer from, such as a bare statement whose result is immediately discarded, or when inference would otherwise pick `Object`:

```java
// No assignment target - the compiler can't infer T without help
List<String> empty = Collections.<String>emptyList();
process(empty);

// Ambiguous overloads sometimes need a witness to select the right one
Collections.<Number>emptyList().forEach(System.out::println);
```

Keep these rare cases isolated and prefer restructuring the call so a target type is available instead of reaching for a witness by default.

## See Also

- [`type-var-inference-readability`](type-var-inference-readability.md) - Local variable inference interacts with generic method inference
- [`type-bounded-wildcards-pecs`](type-bounded-wildcards-pecs.md) - Wildcards affect what the compiler can infer
- [`modern-var-local-inference`](modern-var-local-inference.md) - `var` relies on the same inference machinery
