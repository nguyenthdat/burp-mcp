# modern-sequenced-collections

> Use `SequencedCollection`/`SequencedMap`/`SequencedSet` APIs

## Why It Matters

Before Java 21, getting the first or last element of a `LinkedHashSet` or reversing a `List` required inconsistent, collection-specific idioms (`list.get(list.size() - 1)` vs. iterator gymnastics for a `LinkedHashSet`), and there was no common interface expressing "this collection has a defined encounter order." JEP 431's sequenced collection interfaces (`SequencedCollection`, `SequencedSet`, `SequencedMap`) unify first/last access, reversed views, and insertion order across `List`, `Deque`, `LinkedHashSet`, and `LinkedHashMap`.

## Bad

```java
List<String> names = new ArrayList<>(List.of("Alice", "Bob", "Carol"));

// Verbose, error-prone index math for first/last access
String first = names.get(0);
String last = names.get(names.size() - 1);

// Reversing required a copy or a manual loop
List<String> reversed = new ArrayList<>(names);
Collections.reverse(reversed);

// LinkedHashSet had no clean way to get the first/last element at all
LinkedHashSet<String> visited = new LinkedHashSet<>(List.of("a", "b", "c"));
String firstVisited = visited.iterator().next(); // clunky, and no "last" equivalent
```

## Good

```java
List<String> names = new ArrayList<>(List.of("Alice", "Bob", "Carol"));

String first = names.getFirst();
String last = names.getLast();

List<String> reversed = names.reversed(); // live reversed view, no copy needed

LinkedHashSet<String> visited = new LinkedHashSet<>(List.of("a", "b", "c"));
String firstVisited = visited.getFirst();
String lastVisited = visited.getLast();
visited.addFirst("z"); // now also supported directly
```

## Sequenced Maps

`LinkedHashMap` and other ordered maps now implement `SequencedMap`, exposing first/last entries directly:

```java
LinkedHashMap<String, Integer> scores = new LinkedHashMap<>();
scores.put("Alice", 90);
scores.put("Bob", 85);

Map.Entry<String, Integer> topEntry = scores.firstEntry();
Map.Entry<String, Integer> lastEntry = scores.lastEntry();
SequencedMap<String, Integer> reversedScores = scores.reversed();
```

`reversed()` returns a view backed by the original collection, not a copy - mutating the view mutates the source, and vice versa. Keep this in mind when passing a `.reversed()` result to code that assumes an independent snapshot.

## See Also

- [`coll-choose-right-collection`](coll-choose-right-collection.md) - Choosing between `List`, `Deque`, and `LinkedHashSet` implementations
- [`coll-immutable-factories`](coll-immutable-factories.md) - `List.of()` results also implement `SequencedCollection`, but reject mutation
- [`modern-records-immutable-data`](modern-records-immutable-data.md) - Newer JDK APIs favor explicit, purpose-built interfaces like this one
