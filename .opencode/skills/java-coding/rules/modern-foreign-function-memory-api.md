# modern-foreign-function-memory-api

> Use the Foreign Function & Memory API instead of JNI

## Why It Matters

JNI requires writing and maintaining separate native glue code (C/C++ headers, `javah`/`javac -h` generated stubs, manual reference management), which is verbose, error-prone, and a common source of native-memory crashes that bypass the JVM's safety guarantees entirely. The Foreign Function & Memory API (finalized in Java 22 via JEP 454, building on JEP 442/419/412/389/393) lets Java code call native libraries and manage off-heap memory directly from pure Java, with the JVM enforcing safe access to that memory.

## Bad

```java
// JNI approach: requires a separate native library, hand-written glue code,
// and a build step to compile a shared object (e.g. libmath.so) alongside the JAR
public class NativeMath {
    static {
        System.loadLibrary("math"); // fragile: platform-specific binary must be on java.library.path
    }

    // Native method - implementation lives in a separate C file, invisible to the JVM's safety checks
    public static native double sqrt(double value);
}

// C side (native_math.c) - manually maintained, easy to introduce a crash or leak here
// JNIEXPORT jdouble JNICALL Java_NativeMath_sqrt(JNIEnv *env, jclass cls, jdouble value) {
//     return sqrt(value);
// }
```

## Good

```java
// Foreign Function & Memory API - finalized in JDK 22 (JEP 454).
// Confirm your project's JDK version; on JDK 21 this API is still in preview/incubator form.
import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

public class NativeMath {
    private static final MethodHandle SQRT;

    static {
        Linker linker = Linker.nativeLinker();
        SymbolLookup stdlib = linker.defaultLookup();
        MemorySegment sqrtAddr = stdlib.find("sqrt").orElseThrow();
        SQRT = linker.downcallHandle(
                sqrtAddr,
                FunctionDescriptor.of(ValueLayout.JAVA_DOUBLE, ValueLayout.JAVA_DOUBLE));
    }

    public static double sqrt(double value) throws Throwable {
        return (double) SQRT.invoke(value); // calls the platform's libc sqrt directly, no native glue code
    }
}
```

## Managing Off-Heap Memory Safely

The API also replaces the older, unsafe `sun.misc.Unsafe`-based off-heap memory tricks with a bounds-checked, lifetime-scoped `MemorySegment`:

```java
try (Arena arena = Arena.ofConfined()) {
    MemorySegment segment = arena.allocate(ValueLayout.JAVA_INT.byteSize() * 100);
    for (int i = 0; i < 100; i++) {
        segment.setAtIndex(ValueLayout.JAVA_INT, i, i * i);
    }
    int fiftieth = segment.getAtIndex(ValueLayout.JAVA_INT, 50);
} // memory is deterministically freed when the arena closes; access after close throws
```

Accessing a `MemorySegment` after its owning `Arena` is closed throws `IllegalStateException` rather than corrupting memory or crashing the JVM, which is the core safety win over both JNI and `Unsafe`.

## See Also

- [`err-try-with-resources`](err-try-with-resources.md) - `Arena` is used with try-with-resources exactly like other closeable resources
- [`perf-avoid-reflection-hot-path`](perf-avoid-reflection-hot-path.md) - `MethodHandle` invocation has different performance characteristics worth profiling
- [`modern-virtual-threads-jep444`](modern-virtual-threads-jep444.md) - Both features shipped as part of the same wave of modern JDK evolution
