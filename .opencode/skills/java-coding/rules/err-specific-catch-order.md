# err-specific-catch-order

> Catch specific exceptions before general ones; use multi-catch

## Why It Matters

The compiler enforces that a more general catch block cannot appear before a more specific one for the same try (it's a compile error for a strict subtype relationship), but a common near-miss is writing near-duplicate catch blocks with identical bodies instead of using multi-catch, or ordering unrelated exceptions in a confusing sequence that misleads readers about which failure is handled how. Ordering from most specific to least specific, and merging identical handling logic with `|`, keeps the intent of each branch clear.

## Bad

```java
public void loadUserData(String userId) {
    try {
        String raw = Files.readString(Path.of(userId + ".json"));
        User user = mapper.readValue(raw, User.class);
        process(user);
    } catch (IOException e) {
        log.error("failed to read or parse user file", e);
        throw new DataLoadException("could not load user " + userId, e);
    } catch (JsonProcessingException e) {
        // Duplicated handling logic - JsonProcessingException IS-A IOException,
        // so this branch is unreachable and this is actually a compile error in Java
        log.error("failed to parse user file", e);
        throw new DataLoadException("could not load user " + userId, e);
    }
}
```

## Good

```java
public void loadUserData(String userId) {
    try {
        String raw = Files.readString(Path.of(userId + ".json"));
        User user = mapper.readValue(raw, User.class);
        process(user);
    } catch (JsonProcessingException e) {
        // More specific subtype of IOException - must come first
        log.error("malformed user file for {}", userId, e);
        throw new DataLoadException("invalid user data for " + userId, e);
    } catch (IOException e) {
        log.error("failed to read user file for {}", userId, e);
        throw new DataLoadException("could not read user " + userId, e);
    }
}
```

## Multi-Catch For Identical Handling

```java
public Config parseConfig(String raw) {
    try {
        return objectMapper.readValue(raw, Config.class);
    } catch (JsonParseException | JsonMappingException e) {
        // Unrelated exception types, same handling - combine with |
        throw new ConfigParseException("invalid config format", e);
    }
}
```

## See Also

- [`err-no-catch-broad`](err-no-catch-broad.md) - Don't catch `Exception`/`Throwable` broadly
- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - Build a custom exception hierarchy for domain errors
- [`err-exception-chaining`](err-exception-chaining.md) - Chain causes via constructor
