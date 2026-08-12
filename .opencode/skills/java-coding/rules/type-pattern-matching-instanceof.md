# type-pattern-matching-instanceof

> Use pattern matching for `instanceof`

## Why It Matters

The classic `instanceof` + cast idiom repeats the type name three times and introduces a variable whose scope is easy to get wrong, inviting `ClassCastException` if the cast is copy-pasted to the wrong branch. Pattern matching for `instanceof` (finalized in Java 16) binds the cast variable directly in the condition, eliminating the redundant cast and narrowing the variable's scope to exactly where it is proven safe.

## Bad

```java
public double area(Object shape) {
    if (shape instanceof Circle) {
        Circle c = (Circle) shape; // redundant cast, easy to typo the type
        return Math.PI * c.radius() * c.radius();
    } else if (shape instanceof Rectangle) {
        Rectangle r = (Rectangle) shape;
        return r.width() * r.height();
    }
    throw new IllegalArgumentException("Unknown shape: " + shape);
}

public boolean sameLength(Object a, Object b) {
    if (a instanceof String && b instanceof String) {
        String sa = (String) a; // two casts to remember to keep in sync
        String sb = (String) b;
        return sa.length() == sb.length();
    }
    return false;
}
```

## Good

```java
public double area(Object shape) {
    if (shape instanceof Circle c) {
        return Math.PI * c.radius() * c.radius();
    } else if (shape instanceof Rectangle r) {
        return r.width() * r.height();
    }
    throw new IllegalArgumentException("Unknown shape: " + shape);
}

public boolean sameLength(Object a, Object b) {
    return a instanceof String sa && b instanceof String sb
            && sa.length() == sb.length();
}
```

## Flow Scoping

The pattern variable is only in scope where the compiler can prove the `instanceof` was true, which enables useful negated forms:

```java
public void process(Object obj) {
    if (!(obj instanceof String s)) {
        return; // early exit
    }
    // s is in scope here - flow analysis proves obj is a String past this point
    System.out.println(s.toUpperCase());
}
```

This pattern (early-return guard + fall-through binding) is idiomatic in modern Java and reads better than nesting the happy path inside an `if` block.

## See Also

- [`type-pattern-matching-switch`](type-pattern-matching-switch.md) - The same pattern-binding idea extended to `switch`
- [`modern-record-deconstruction-patterns`](modern-record-deconstruction-patterns.md) - Pattern matching can also deconstruct record components
- [`anti-instanceof-chain-instead-polymorphism`](anti-instanceof-chain-instead-polymorphism.md) - When a chain of `instanceof` checks signals a deeper design problem
