# coll-choose-right-collection

> Choose the right collection type for the access pattern

## Why It Matters

`ArrayList`, `LinkedList`, `HashSet`, `TreeSet`, `HashMap`, and `ArrayDeque` have wildly different performance characteristics for insertion, removal, lookup, and iteration. Picking the default (`ArrayList`, `HashMap`) without thinking about the actual access pattern leads to O(n) operations where O(1) or O(log n) was available, or wastes memory on structures with overhead the use case never needed.

## Bad

```java
// Frequent removals from the front - ArrayList shifts every remaining element
List<Task> queue = new ArrayList<>();
queue.add(new Task("build"));
queue.add(new Task("test"));
while (!queue.isEmpty()) {
    Task next = queue.remove(0); // O(n) shift on every call
    process(next);
}

// Membership checks on a List - O(n) per lookup
List<String> allowedUsers = new ArrayList<>(loadUsers());
if (allowedUsers.contains(currentUser)) { // O(n) scan
    grantAccess();
}

// Needs sorted iteration order but uses HashMap
Map<String, Integer> scoresByName = new HashMap<>();
scoresByName.put("carol", 92);
scoresByName.put("alice", 88);
scoresByName.put("bob", 95);
// Iteration order is unspecified - can't rely on it for a leaderboard
for (var entry : scoresByName.entrySet()) {
    System.out.println(entry.getKey() + ": " + entry.getValue());
}
```

## Good

```java
// FIFO access pattern - use ArrayDeque, O(1) at both ends
Deque<Task> queue = new ArrayDeque<>();
queue.add(new Task("build"));
queue.add(new Task("test"));
while (!queue.isEmpty()) {
    Task next = queue.poll(); // O(1)
    process(next);
}

// Membership checks - use a Set, O(1) average lookup
Set<String> allowedUsers = new HashSet<>(loadUsers());
if (allowedUsers.contains(currentUser)) {
    grantAccess();
}

// Sorted iteration order required - use TreeMap
Map<String, Integer> scoresByName = new TreeMap<>();
scoresByName.put("carol", 92);
scoresByName.put("alice", 88);
scoresByName.put("bob", 95);
// Iterates alice, bob, carol - guaranteed by the TreeMap contract
for (var entry : scoresByName.entrySet()) {
    System.out.println(entry.getKey() + ": " + entry.getValue());
}
```

## Decision Guide

| Need | Use |
|------|-----|
| Random access by index | `ArrayList` |
| Frequent insert/remove at both ends | `ArrayDeque` |
| Frequent insert/remove in the middle, sequential access only | `LinkedList` (rare - usually still `ArrayList`) |
| Uniqueness, no order guarantee | `HashSet` |
| Uniqueness, insertion order preserved | `LinkedHashSet` |
| Uniqueness, sorted order | `TreeSet` |
| Key lookup, no order guarantee | `HashMap` |
| Key lookup, insertion order preserved | `LinkedHashMap` |
| Key lookup, sorted by key | `TreeMap` |

## See Also

- [`coll-avoid-legacy-classes`](coll-avoid-legacy-classes.md) - Avoid `Vector`/`Hashtable`/`Stack` in favor of modern replacements
- [`coll-immutable-factories`](coll-immutable-factories.md) - Use immutable factories when the collection never needs to change
- [`perf-avoid-autoboxing-hot-path`](perf-avoid-autoboxing-hot-path.md) - Related performance concern for primitive-heavy collections
- [`conc-concurrent-collections`](conc-concurrent-collections.md) - Choosing collections under concurrent access
