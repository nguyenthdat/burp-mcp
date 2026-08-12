# coll-immutable-factories

> Use `List.of`/`Map.of`/`Set.of` immutable factories

## Why It Matters

`List.of`, `Map.of`, and `Set.of` (Java 9+) produce truly immutable collections with no defensive copying overhead at the call site and no accidental mutation risk. Compared to `Arrays.asList` (fixed-size but mutable elements, backed by an array) or a `new ArrayList<>()` left mutable by convention, they communicate intent directly in the type and fail fast on any mutation attempt.

## Bad

```java
// Arrays.asList - looks immutable but isn't; set() works, add()/remove() throw at runtime
List<String> statuses = Arrays.asList("PENDING", "ACTIVE", "CLOSED");
statuses.set(0, "UNKNOWN"); // silently allowed, mutates the backing array

// Mutable list built and shared as if it were a constant
public static List<String> DEFAULT_ROLES = new ArrayList<>();
static {
    DEFAULT_ROLES.add("USER");
    DEFAULT_ROLES.add("GUEST");
}
// Any caller can do DEFAULT_ROLES.add("ADMIN") and corrupt shared state

// Map built with put() calls when it never changes after construction
Map<String, Integer> httpStatusNames = new HashMap<>();
httpStatusNames.put(200, 200); // wrong types caught only at compile errors like this
```

## Good

```java
// Truly immutable - throws UnsupportedOperationException on any mutation attempt
List<String> statuses = List.of("PENDING", "ACTIVE", "CLOSED");

// Immutable constant, safe to expose as public static
public static final List<String> DEFAULT_ROLES = List.of("USER", "GUEST");

// Map.of for small fixed maps
Map<Integer, String> httpStatusNames = Map.of(
    200, "OK",
    404, "Not Found",
    500, "Internal Server Error"
);

// Set.of for fixed membership checks
Set<String> reservedNames = Set.of("admin", "root", "system");
```

## Beyond a Handful of Entries

`Map.of`/`Set.of` have overloads only up to 10 key-value pairs. For larger fixed collections, use `Map.ofEntries` or build then wrap with `copyOf`:

```java
Map<String, Integer> countryCodes = Map.ofEntries(
    Map.entry("US", 1),
    Map.entry("GB", 44),
    Map.entry("JP", 81),
    Map.entry("DE", 49)
    // ... more entries
);

// Or freeze a dynamically built collection
List<String> collected = new ArrayList<>();
collected.add("a");
collected.add("b");
List<String> frozen = List.copyOf(collected);
```

## Null Handling

Unlike `Arrays.asList`, the `of()` factories reject `null` elements with an immediate `NullPointerException` at construction time rather than allowing a null to leak into the collection and fail later at an unrelated call site.

## See Also

- [`coll-collection-factory-vs-loop`](coll-collection-factory-vs-loop.md) - Prefer factories over manual population loops
- [`coll-unmodifiable-view`](coll-unmodifiable-view.md) - Wrapping existing mutable collections instead of copying
- [`null-empty-collection-not-null`](null-empty-collection-not-null.md) - Related null-avoidance pattern for collections
- [`api-immutable-by-default`](api-immutable-by-default.md) - Broader immutability philosophy for API design
