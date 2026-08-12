# api-immutable-by-default

> Design objects immutable by default

## Why It Matters

Mutable objects can be changed out from under any code holding a reference to them, which makes reasoning about correctness in concurrent code nearly impossible and turns bugs like "why did this list change" into multi-hour debugging sessions. Immutable objects are inherently thread-safe, safe to share and cache without copying, and easier to reason about because their state never diverges from what the constructor established.

## Bad

```java
public class Account {
    private String owner;
    private double balance;
    private List<String> transactionLog;

    public Account(String owner, double balance) {
        this.owner = owner;
        this.balance = balance;
        this.transactionLog = new ArrayList<>();
    }

    public void setBalance(double balance) { this.balance = balance; } // anyone can mutate
    public void setOwner(String owner) { this.owner = owner; }
    public List<String> getTransactionLog() { return transactionLog; } // leaks mutable internal list
}

// Shared across threads without synchronization - a data race waiting to happen
Account shared = new Account("Alice", 100.0);
shared.getTransactionLog().add("unauthorized entry"); // mutated from outside!
shared.setBalance(-500);                              // no validation, no audit trail
```

## Good

```java
public final class Account {
    private final String owner;
    private final double balance;
    private final List<String> transactionLog;

    public Account(String owner, double balance) {
        this(owner, balance, List.of());
    }

    private Account(String owner, double balance, List<String> transactionLog) {
        this.owner = Objects.requireNonNull(owner);
        this.balance = balance;
        this.transactionLog = List.copyOf(transactionLog); // defensive, unmodifiable
    }

    public String owner() { return owner; }
    public double balance() { return balance; }
    public List<String> transactionLog() { return transactionLog; } // safe to expose - immutable

    // State "changes" produce a new instance instead of mutating this one
    public Account withDeposit(double amount) {
        if (amount <= 0) {
            throw new IllegalArgumentException("Deposit must be positive");
        }
        List<String> newLog = new ArrayList<>(transactionLog);
        newLog.add("deposit: " + amount);
        return new Account(owner, balance + amount, newLog);
    }
}

// Safe to share freely - no defensive copying, no synchronization needed
Account original = new Account("Alice", 100.0);
Account afterDeposit = original.withDeposit(50.0);
// original is untouched; afterDeposit is a distinct, fully valid snapshot
```

## When Mutability Is Justified

Mutable builders, caches with bounded lifetimes, and performance-critical hot loops that would otherwise allocate excessively are reasonable exceptions — but confine the mutable state to a narrow, well-documented scope (e.g. a `Builder` inner class) rather than letting it leak into the public object model.

## See Also

- [`api-record-data-carrier`](api-record-data-carrier.md) - Records make immutable data carriers concise
- [`api-defensive-copy-mutable-args`](api-defensive-copy-mutable-args.md) - Preventing external mutation from leaking in
- [`conc-immutable-thread-safety`](conc-immutable-thread-safety.md) - Why immutability is a concurrency strategy
- [`null-defensive-copy`](null-defensive-copy.md) - Copying collections defensively at API boundaries
