# api-interface-default-methods

> Prefer interfaces with default methods over rigid abstract base classes

## Why It Matters

An abstract base class forces implementers into a single inheritance slot and couples them to its state and constructor chain, while an interface with default methods lets a class implement any number of contracts and only override what's actually specific to it. Default methods let library authors add new behavior to an interface after release without breaking every existing implementer — something an abstract class cannot do without a recompile of every subclass.

## Bad

```java
// Forces every implementer to extend this specific class,
// burning their one inheritance slot and inheriting unwanted state
public abstract class AbstractValidator {
    protected List<String> errors = new ArrayList<>();

    public abstract boolean isValid(String input);

    public String describeErrors() {
        return String.join(", ", errors);
    }
}

public class EmailValidator extends AbstractValidator {
    public boolean isValid(String input) {
        boolean ok = input.contains("@");
        if (!ok) errors.add("missing @");
        return ok;
    }
}

// EmailValidator can no longer extend anything else, e.g. a shared base
// for framework integration - the slot is already spent
```

## Good

```java
public interface Validator {
    boolean isValid(String input);

    // Default method: implementers get this for free but can override it
    default String describe() {
        return "Validator[" + getClass().getSimpleName() + "]";
    }

    // Composable default: combine validators without inheritance
    default Validator and(Validator other) {
        return input -> this.isValid(input) && other.isValid(input);
    }
}

public class EmailValidator implements Validator {
    public boolean isValid(String input) {
        return input.contains("@");
    }
}

public class LengthValidator implements Validator {
    private final int maxLength;
    public LengthValidator(int maxLength) { this.maxLength = maxLength; }

    public boolean isValid(String input) {
        return input.length() <= maxLength;
    }
}

// Free to implement multiple interfaces and compose behavior
Validator combined = new EmailValidator().and(new LengthValidator(50));
```

## Evolving an Interface Without Breaking Implementers

```java
public interface Repository<T, ID> {
    Optional<T> findById(ID id);
    void save(T entity);

    // Added later - existing implementers compile unchanged
    default List<T> findAllById(List<ID> ids) {
        return ids.stream().map(this::findById)
                .flatMap(Optional::stream)
                .toList();
    }
}
```

## When an Abstract Class Is Still Right

Use an abstract class when subclasses must share actual mutable state or a mandatory constructor-enforced invariant (e.g. a template-method base class that guarantees setup/teardown ordering) — something an interface, which cannot hold instance state, cannot express.

## See Also

- [`api-composition-over-inheritance`](api-composition-over-inheritance.md) - Preferring delegation over any inheritance at all
- [`api-final-classes-not-designed-for-inheritance`](api-final-classes-not-designed-for-inheritance.md) - Closing off classes that aren't meant as bases
- [`type-generic-method-inference`](type-generic-method-inference.md) - Generic default methods and type inference interactions
- [`api-sealed-closed-hierarchy`](api-sealed-closed-hierarchy.md) - Sealing an interface hierarchy when the variant set is fixed
