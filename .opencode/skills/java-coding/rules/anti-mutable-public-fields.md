# anti-mutable-public-fields

> Don't expose mutable public fields

## Why It Matters

A public non-final field lets any code anywhere in the program mutate an object's state without going through validation, invariant checks, or even a debugger breakpoint you can set on a setter. It also locks you into the field's exact representation forever, since any later refactor to a method breaks every caller's source compatibility.

## Bad

```java
public class Account {
  public double balance;      // Anyone can set this to a negative number
  public List<String> tags;   // Anyone can mutate the list directly, bypassing any bookkeeping
}

Account account = new Account();
account.balance = -500;              // No validation, silently corrupts invariants
account.tags.add(null);              // Bypasses any tag-validation logic that might exist
```

## Good

```java
public final class Account {
  private double balance;
  private final List<String> tags = new ArrayList<>();

  public double getBalance() {
    return balance;
  }

  public void deposit(double amount) {
    if (amount <= 0) {
      throw new IllegalArgumentException("deposit amount must be positive");
    }
    balance += amount;
  }

  public List<String> getTags() {
    return List.copyOf(tags); // Defensive copy - caller can't mutate internal state
  }

  public void addTag(String tag) {
    tags.add(Objects.requireNonNull(tag, "tag must not be null"));
  }
}
```

## When a Public Field Is Acceptable

```java
// Public static final constants are fine - they are truly immutable
public static final int MAX_RETRIES = 3;

// Records expose fields via generated accessors, not raw public fields,
// and their components are implicitly final - this is the idiomatic
// "data carrier" pattern, not the anti-pattern.
public record Point(int x, int y) { }
```

## See Also

- [`api-immutable-by-default`](api-immutable-by-default.md) - The positive rule for designing state that can't be corrupted externally
- [`api-defensive-copy-mutable-args`](api-defensive-copy-mutable-args.md) - Applying the same discipline to constructor/method arguments
- [`api-record-data-carrier`](api-record-data-carrier.md) - The correct way to expose plain data without mutable public fields
- [`anti-anemic-domain-model`](anti-anemic-domain-model.md) - A related failure mode where getters/setters replace real invariants
