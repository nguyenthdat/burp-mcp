# coll-collection-factory-vs-loop

> Prefer `of()`/`copyOf()` factories over manual population loops

## Why It Matters

Manually constructing a collection with a `new` call followed by a sequence of `add` calls is more verbose than the equivalent factory call, and it leaves the collection mutable when it was only ever meant to hold a fixed set of values. `List.of`/`Set.of`/`Map.of` and their `copyOf` counterparts state the entire contents in one expression and return an immutable result by default.

## Bad

```java
// Manual population loop for a fixed, known-at-compile-time set of values
List<String> weekdays = new ArrayList<>();
weekdays.add("MON");
weekdays.add("TUE");
weekdays.add("WED");
weekdays.add("THU");
weekdays.add("FRI");

// Copying an existing collection via a manual loop
List<String> source = fetchNames();
List<String> copy = new ArrayList<>();
for (String name : source) {
    copy.add(name);
}

// Building a fixed lookup map with repeated put() calls
Map<Integer, String> grades = new HashMap<>();
grades.put(90, "A");
grades.put(80, "B");
grades.put(70, "C");
```

## Good

```java
// Single expression, immutable by default
List<String> weekdays = List.of("MON", "TUE", "WED", "THU", "FRI");

// copyOf expresses "snapshot this collection" directly
List<String> source = fetchNames();
List<String> copy = List.copyOf(source);

// Map.of for a small fixed lookup table
Map<Integer, String> grades = Map.of(90, "A", 80, "B", 70, "C");
```

## When a Loop Is Still Correct

Use a loop (or a stream collector) when the contents are computed conditionally, come from an external source at runtime, or the resulting collection genuinely needs to remain mutable for later use:

```java
// Contents depend on runtime data - a factory call can't express this
List<String> activeUsers = new ArrayList<>();
for (User user : allUsers) {
    if (user.isActive()) {
        activeUsers.add(user.name());
    }
}
// Prefer the stream equivalent when this shape appears:
List<String> activeUsersStream = allUsers.stream()
    .filter(User::isActive)
    .map(User::name)
    .toList();
```

## `copyOf` Skips Redundant Copies

`List.copyOf`, `Set.copyOf`, and `Map.copyOf` are specified to return the source itself (without copying) if it is already an unmodifiable collection of the matching type, so calling `copyOf` defensively at an API boundary carries no cost when the caller already passed an immutable value.

## See Also

- [`coll-immutable-factories`](coll-immutable-factories.md) - The immutable factory methods this rule builds on
- [`coll-unmodifiable-view`](coll-unmodifiable-view.md) - Choosing between a live view and a snapshot copy
- [`coll-stream-for-transformation`](coll-stream-for-transformation.md) - Building collections from computed/filtered data instead of literals
- [`api-static-factory-over-constructor`](api-static-factory-over-constructor.md) - The broader static-factory-method principle
