# perf-string-intern-caution

> Use `String.intern()` cautiously and deliberately

## Why It Matters

`String.intern()` moves a string into the JVM's string pool so identical values share one instance, which can reduce memory when you have massive numbers of duplicate strings. But the pool historically lived in a fixed-size, GC-sensitive region, and even on modern JVMs interning has real lookup and synchronization cost. Calling it reflexively on every parsed string - rather than only on genuinely long-lived, highly duplicated values - trades a small, well-understood cost (extra `String` objects) for a global lock contention point and potential pool bloat.

## Bad

```java
public List<String> parseCountryCodes(List<String> rawLines) {
    List<String> codes = new ArrayList<>(rawLines.size());
    for (String line : rawLines) {
        codes.add(line.trim().intern());  // Interning every line, including one-off garbage
    }
    return codes;
}

public String normalize(String input) {
    return input.toLowerCase().intern();  // Reflex call; most inputs are never duplicated
}
```

## Good

```java
public List<String> parseCountryCodes(List<String> rawLines) {
    // Country codes: small, fixed, highly-duplicated set - a great fit for interning
    List<String> codes = new ArrayList<>(rawLines.size());
    for (String line : rawLines) {
        codes.add(line.trim().intern());
    }
    return codes;
}

public String normalize(String input) {
    return input.toLowerCase();  // No interning; duplication is not expected here
}

// Alternative: build your own bounded cache for domain-specific interning
// when you need control the JVM's global pool doesn't give you.
public final class SymbolInterner {
    private final Map<String, String> pool = new ConcurrentHashMap<>();

    public String intern(String value) {
        return pool.computeIfAbsent(value, Function.identity());
    }
}
```

## When Interning Pays Off

```java
// Parsing a huge log file where the same handful of log-level strings
// ("INFO", "WARN", "ERROR") repeat millions of times.
String level = parseLevel(line).intern();  // Collapses to a handful of shared instances
```

## See Also

- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`perf-collection-sizing`](perf-collection-sizing.md) - Size collections up front when the count is known
- [`perf-profile-before-optimizing`](perf-profile-before-optimizing.md) - Profile before optimizing
