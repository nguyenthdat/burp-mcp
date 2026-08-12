# null-defensive-copy

> Defensive-copy mutable fields to prevent aliasing/null surprises

## Why It Matters

If a constructor or getter hands out a direct reference to an internal mutable collection or array, any external code can mutate it — or replace entries with `null` — behind the object's back, silently breaking invariants the class thinks it enforces. Defensive copying on the way in and the way out closes that hole and keeps `null` from leaking into state the class assumed was validated.

## Bad

```java
public class Schedule {

    private final List<Meeting> meetings;

    public Schedule(List<Meeting> meetings) {
        this.meetings = meetings; // caller's list, not ours - they can mutate it later
    }

    public List<Meeting> getMeetings() {
        return meetings; // caller can add(null) or clear() our internal state
    }
}

List<Meeting> shared = new ArrayList<>(List.of(standup));
Schedule schedule = new Schedule(shared);
shared.add(null);              // corrupts schedule's internal state too
schedule.getMeetings().clear(); // wipes it out entirely
```

## Good

```java
public class Schedule {

    private final List<Meeting> meetings;

    public Schedule(List<Meeting> meetings) {
        // Copy on the way in, reject nulls up front
        this.meetings = List.copyOf(meetings); // also unmodifiable and null-hostile
    }

    public List<Meeting> getMeetings() {
        return meetings; // already immutable, safe to hand out directly
    }
}

List<Meeting> shared = new ArrayList<>(List.of(standup));
Schedule schedule = new Schedule(shared);
shared.add(null);                // schedule is unaffected
// schedule.getMeetings().add(x); // throws UnsupportedOperationException - caught immediately
```

## Arrays Need Explicit Cloning

```java
public final class Fingerprint {

    private final byte[] bytes;

    public Fingerprint(byte[] bytes) {
        this.bytes = bytes.clone(); // arrays are always mutable, always copy
    }

    public byte[] bytes() {
        return bytes.clone(); // never leak the internal array
    }
}
```

## See Also

- [`null-empty-collection-not-null`](null-empty-collection-not-null.md) - Return empty collections, never `null`
- [`api-defensive-copy-mutable-args`](api-defensive-copy-mutable-args.md) - Defensive-copy mutable constructor arguments
- [`api-immutable-by-default`](api-immutable-by-default.md) - Design classes to be immutable by default
- [`coll-immutable-factories`](coll-immutable-factories.md) - Use `List.of`/`Set.of`/`Map.of`/`List.copyOf`
