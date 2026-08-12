# api-defensive-copy-mutable-args

> Defensive-copy mutable constructor arguments

## Why It Matters

If a constructor stores a reference to a caller-supplied mutable object (a `List`, `Date`, array, or similar) instead of copying it, the caller retains the ability to mutate your "immutable" object's internal state after construction, silently violating any invariant you thought you had established. This class of bug is especially dangerous because it breaks encapsulation invisibly — nothing in the constructing code looks wrong, and the mutation can happen far away, at a completely different time.

## Bad

```java
public final class Schedule {
    private final List<LocalDate> blackoutDates;

    public Schedule(List<LocalDate> blackoutDates) {
        this.blackoutDates = blackoutDates; // stores the caller's reference directly
    }

    public boolean isBlackout(LocalDate date) {
        return blackoutDates.contains(date);
    }
}

List<LocalDate> dates = new ArrayList<>(List.of(LocalDate.of(2026, 12, 25)));
Schedule schedule = new Schedule(dates);

dates.clear(); // mutating the original list...
dates.add(LocalDate.of(2026, 1, 1));

schedule.isBlackout(LocalDate.of(2026, 12, 25)); // false! silently changed underneath us
```

## Good

```java
public final class Schedule {
    private final List<LocalDate> blackoutDates;

    public Schedule(List<LocalDate> blackoutDates) {
        // Copy before storing - later external mutation cannot reach our state
        this.blackoutDates = List.copyOf(blackoutDates);
    }

    public boolean isBlackout(LocalDate date) {
        return blackoutDates.contains(date);
    }
}

List<LocalDate> dates = new ArrayList<>(List.of(LocalDate.of(2026, 12, 25)));
Schedule schedule = new Schedule(dates);

dates.clear();
dates.add(LocalDate.of(2026, 1, 1));

schedule.isBlackout(LocalDate.of(2026, 12, 25)); // true - unaffected by external mutation
```

## Defensive Copy on the Way Out Too

```java
public final class Portfolio {
    private final List<String> holdings;

    public Portfolio(List<String> holdings) {
        this.holdings = new ArrayList<>(holdings); // copy on the way in
    }

    public List<String> holdings() {
        return List.copyOf(holdings); // and copy (or wrap unmodifiable) on the way out
    }
}
```

Without the copy on the accessor, `portfolio.holdings().add(...)` would either throw (if backed by an immutable list already) or, worse, silently mutate the object's real internal state if the accessor returned it directly.

## When Copying Can Be Skipped

Skip the copy when the argument type is genuinely immutable (`String`, boxed primitives, records composed entirely of immutable fields, or results of `List.of(...)`/`Map.of(...)`), or when the constructor explicitly documents that it takes ownership of the passed-in mutable object and the caller must not retain it.

## See Also

- [`api-immutable-by-default`](api-immutable-by-default.md) - The broader design goal this practice supports
- [`null-defensive-copy`](null-defensive-copy.md) - Defensive copying in the context of null-safety
- [`coll-immutable-factories`](coll-immutable-factories.md) - `List.copyOf`, `Map.copyOf`, and friends
- [`api-record-compact-constructor-validation`](api-record-compact-constructor-validation.md) - Copying and validating together in a record's compact constructor
