# perf-avoid-reflection-hot-path

> Avoid reflection in hot paths

## Why It Matters

Reflective calls bypass the JIT's ability to inline and devirtualize method invocations, involve access checks, and often allocate argument arrays for every call. Using `Method.invoke` or field reflection inside a loop that runs on every request can be an order of magnitude slower than a direct call or a cached `MethodHandle`. Reflection is fine for one-time setup (wiring frameworks, plugin discovery) but should never sit in the per-request critical path.

## Bad

```java
public class JsonMapper {
    public Object getField(Object bean, String fieldName) {
        try {
            Field field = bean.getClass().getDeclaredField(fieldName);  // Looked up every call
            field.setAccessible(true);
            return field.get(bean);  // Reflective read on every request
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }
}

public void dispatch(Object handler, String methodName, Object arg) throws Exception {
    Method method = handler.getClass().getMethod(methodName, Object.class);  // Repeated lookup
    method.invoke(handler, arg);  // Invoked per event, in the hot path
}
```

## Good

```java
public class JsonMapper {
    // Cache resolved accessors once per class, not once per call
    private final Map<Class<?>, Map<String, VarHandle>> handleCache = new ConcurrentHashMap<>();

    public Object getField(Object bean, String fieldName) {
        VarHandle handle = handleCache
            .computeIfAbsent(bean.getClass(), this::resolveFields)
            .get(fieldName);
        return handle.get(bean);
    }

    private Map<String, VarHandle> resolveFields(Class<?> type) {
        // Expensive reflective lookup happens once, at cache-miss time
        Map<String, VarHandle> result = new HashMap<>();
        for (Field field : type.getDeclaredFields()) {
            try {
                result.put(field.getName(), MethodHandles.privateLookupIn(type, MethodHandles.lookup())
                    .unreflectVarHandle(field));
            } catch (IllegalAccessException e) {
                throw new IllegalStateException(e);
            }
        }
        return result;
    }
}
```

## When Reflection Is Acceptable

```java
// One-time framework wiring at startup - not a hot path.
for (Class<?> pluginClass : scanForPlugins()) {
    Plugin plugin = (Plugin) pluginClass.getDeclaredConstructor().newInstance();
    registry.register(plugin);
}
```

## See Also

- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`perf-profile-before-optimizing`](perf-profile-before-optimizing.md) - Profile before optimizing
- [`perf-jmh-benchmarking`](perf-jmh-benchmarking.md) - Use JMH for reliable microbenchmarks
