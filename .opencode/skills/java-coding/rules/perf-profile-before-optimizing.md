# perf-profile-before-optimizing

> Profile before optimizing

## Why It Matters

Intuition about where an application spends its time is frequently wrong, especially in JVM applications where JIT compilation, garbage collection, and I/O latency interact in non-obvious ways. Optimizing a method that consumes 0.1% of runtime wastes engineering effort and adds risk of regressions, while the actual bottleneck - often a database call, a lock, or GC pauses - goes unaddressed. A profiler turns "I think this is slow" into "this method accounts for 40% of CPU time," which is the only basis for a justified optimization.

## Bad

```java
// Engineer assumes the JSON parsing is the bottleneck because it "feels
// complex," and spends two days hand-writing a custom parser - without
// ever running a profiler to confirm parsing shows up in the hot path at all.
public Response handleRequest(String rawJson) {
    Order order = customFastJsonParser.parse(rawJson);  // "Optimized" based on a guess
    return orderService.process(order);  // The real bottleneck, a synchronous DB call, is untouched
}
```

## Good

```java
// 1. Capture a JDK Flight Recorder profile under realistic load:
//    java -XX:StartFlightRecording=filename=recording.jfr,duration=60s -jar app.jar
//
// 2. Or attach async-profiler for low-overhead sampling:
//    ./profiler.sh -d 30 -f flamegraph.html <pid>
//
// 3. Inspect the flamegraph / JMC report. Suppose it shows:
//    orderService.process()       -> 78% of wall time (blocked on JDBC call)
//    customFastJsonParser.parse() -> 2% of wall time
//
// 4. Optimize the actual bottleneck - e.g. batch the DB calls or add an
//    index - and re-profile to confirm the change moved the needle.
public Response handleRequest(String rawJson) {
    Order order = objectMapper.readValue(rawJson, Order.class);  // Standard parser is fine; not the bottleneck
    return orderService.processBatched(order);  // Addresses the measured hot spot
}
```

## Tools Worth Knowing

```
- JDK Flight Recorder (JFR) + JDK Mission Control - low-overhead, production-safe.
- async-profiler - sampling CPU/allocation profiler, produces flamegraphs.
- JMH - for isolated microbenchmarks once a hot method is identified.
- VisualVM - quick, GUI-based heap/CPU inspection during development.
```

## See Also

- [`perf-avoid-premature-optimization`](perf-avoid-premature-optimization.md) - Don't optimize before profiling
- [`perf-jmh-benchmarking`](perf-jmh-benchmarking.md) - Use JMH for reliable microbenchmarks
- [`perf-avoid-unnecessary-object-creation`](perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
