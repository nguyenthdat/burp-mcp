# type-recursive-generic-bound

> Use recursive generic bounds for self-referencing builders

## Why It Matters

A fluent builder base class needs each chained method to return the *concrete* subtype so subclass-specific methods remain chainable, but a plain return type of the base class loses that information at the point of use. The recursive generic bound idiom (`T extends AbstractBuilder<T>`), sometimes called the "curiously recurring generic pattern," lets each method in the base class return `T` and have it resolve to the caller's actual subtype.

## Bad

```java
abstract class AbstractBuilder {
    private String name;

    public AbstractBuilder name(String name) {
        this.name = name;
        return this; // returns AbstractBuilder, not the real subtype
    }
}

class CarBuilder extends AbstractBuilder {
    private int wheels;

    public CarBuilder wheels(int wheels) {
        this.wheels = wheels;
        return this;
    }
}

// Fails: name() returns AbstractBuilder, which has no wheels() method
CarBuilder car = new CarBuilder()
        .name("Tesla")
        // .wheels(4)   <- compile error, chain is broken after name()
        ;
```

## Good

```java
abstract class AbstractBuilder<T extends AbstractBuilder<T>> {
    private String name;

    @SuppressWarnings("unchecked")
    protected T self() {
        return (T) this; // single, well-understood unchecked cast, isolated here
    }

    public T name(String name) {
        this.name = name;
        return self();
    }
}

class CarBuilder extends AbstractBuilder<CarBuilder> {
    private int wheels;

    public CarBuilder wheels(int wheels) {
        this.wheels = wheels;
        return this;
    }
}

// Chain works across both base and subclass methods, in any order
CarBuilder car = new CarBuilder()
        .name("Tesla")
        .wheels(4);
```

## Deeper Hierarchies

The pattern scales to further subclassing as long as each level re-parameterizes correctly:

```java
class SportsCarBuilder extends CarBuilder {
    // Inherits name() -> CarBuilder and wheels() -> CarBuilder correctly
    // because CarBuilder fixed T = CarBuilder, which is itself and its subclasses
}
```

If a class three levels deep needs its own chainable methods to also return its exact type, it must itself become generic (`SportsCarBuilder<T extends SportsCarBuilder<T>>`) rather than being a concrete leaf - plan the hierarchy depth before choosing this pattern, since it adds real complexity.

## See Also

- [`api-builder-complex-construction`](api-builder-complex-construction.md) - The broader builder pattern this bound supports
- [`api-fluent-method-chaining`](api-fluent-method-chaining.md) - Fluent chaining APIs that benefit from self-typed returns
- [`type-avoid-unchecked-cast`](type-avoid-unchecked-cast.md) - The `self()` cast is a textbook justified unchecked cast
