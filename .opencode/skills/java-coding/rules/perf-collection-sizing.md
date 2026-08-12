# perf-collection-sizing

> Size collections up front when the count is known

## Why It Matters

`ArrayList` and `HashMap` grow by reallocating and copying their backing array whenever they exceed capacity - typically 1.5x for `ArrayList` and 2x for `HashMap`. Constructing one with the default no-arg constructor when you already know (or can estimate) the final size forces avoidable resizes and array copies. Passing an initial capacity is a one-line change that eliminates that churn entirely.

## Bad

```java
public List<String> toUpperCaseAll(List<String> input) {
    List<String> result = new ArrayList<>();  // Starts at capacity 10, resizes repeatedly
    for (String s : input) {
        result.add(s.toUpperCase());  // Triggers reallocation as the list grows past 10, 15, 22...
    }
    return result;
}

public Map<String, User> indexById(List<User> users) {
    Map<String, User> byId = new HashMap<>();  // Default capacity 16, will rehash multiple times
    for (User user : users) {
        byId.put(user.id(), user);
    }
    return byId;
}
```

## Good

```java
public List<String> toUpperCaseAll(List<String> input) {
    List<String> result = new ArrayList<>(input.size());  // Exact size known up front
    for (String s : input) {
        result.add(s.toUpperCase());
    }
    return result;
}

public Map<String, User> indexById(List<User> users) {
    // Account for HashMap's 0.75 load factor to avoid a resize at insert time
    Map<String, User> byId = HashMap.newHashMap(users.size());
    for (User user : users) {
        byId.put(user.id(), user);
    }
    return byId;
}
```

## When the Size Is Unknown

```java
// If the final size is genuinely unpredictable, don't guess wildly - either
// leave the default capacity, or use a conservative estimate based on a
// known lower bound (e.g. one entry per source line, deduped afterward).
List<String> matches = new ArrayList<>();  // Fine: size depends on runtime filtering
for (String line : source) {
    if (pattern.matcher(line).matches()) {
        matches.add(line);
    }
}
```

## See Also

- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`perf-stringbuilder-loop-concat`](perf-stringbuilder-loop-concat.md) - Use `StringBuilder` for string concatenation in loops
- [`coll-choose-right-collection`](coll-choose-right-collection.md) - Choosing the right collection type
- [`coll-collection-factory-vs-loop`](coll-collection-factory-vs-loop.md) - Collection factory methods vs. manual loops
