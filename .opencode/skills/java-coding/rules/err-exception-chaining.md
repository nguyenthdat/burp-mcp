# err-exception-chaining

> Chain causes via constructor, not `initCause` after the fact

## Why It Matters

When you catch an exception and throw a new, more meaningful one without passing the original as the cause, you delete the root-cause stack trace forever — the next person debugging the issue sees only the wrapper exception and has no way to find out what actually failed underneath. Passing the original exception into the new one's constructor (or `initCause`, when the constructor doesn't support it) keeps the full causal chain intact for logs and debuggers.

## Bad

```java
public Config loadConfig(Path path) {
    try {
        return parser.parse(Files.readString(path));
    } catch (IOException e) {
        // Original IOException and its stack trace are gone forever
        throw new ConfigLoadException("failed to load config from " + path);
    }
}
```

## Good

```java
public Config loadConfig(Path path) {
    try {
        return parser.parse(Files.readString(path));
    } catch (IOException e) {
        // Cause preserved - full chain visible in logs and stack traces
        throw new ConfigLoadException("failed to load config from " + path, e);
    }
}

public class ConfigLoadException extends RuntimeException {
    public ConfigLoadException(String message, Throwable cause) {
        super(message, cause);
    }
}
```

## initCause When The Constructor Doesn't Accept A Cause

```java
public Config loadConfig(Path path) throws ReflectiveOperationException {
    try {
        return parser.parse(Files.readString(path));
    } catch (IOException e) {
        // Some exception types (like this hypothetical legacy API) only offer a no-arg
        // constructor; initCause() is the fallback, and can only be called once.
        ReflectiveOperationException wrapped = new ReflectiveOperationException(
                "failed to load config from " + path);
        wrapped.initCause(e);
        throw wrapped;
    }
}
```

## Resulting Stack Trace

```
com.example.ConfigLoadException: failed to load config from /etc/app/config.yaml
    at com.example.ConfigLoader.loadConfig(ConfigLoader.java:15)
Caused by: java.io.IOException: Permission denied
    at java.base/sun.nio.fs.UnixException.translateToIOException(UnixException.java:92)
    ...
```

## See Also

- [`err-unchecked-wrap-checked`](err-unchecked-wrap-checked.md) - Wrap checked exceptions instead of propagating `throws`
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
- [`err-suppressed-exceptions`](err-suppressed-exceptions.md) - Preserve suppressed exceptions from try-with-resources
- [`err-custom-exception-hierarchy`](err-custom-exception-hierarchy.md) - Build a custom exception hierarchy for domain errors
