# proj-flat-small-projects

> Keep small projects flat instead of over-modularizing

## Why It Matters

Multi-module builds, deep package hierarchies, and JPMS module boundaries solve problems that only exist at scale: independent deployability, team ownership boundaries, incremental build times. Imposing that structure on a project with a handful of classes and one deployable artifact adds indirection, boilerplate `build.gradle.kts`/`pom.xml` files, and cross-module wiring that slows down every single change without ever paying for itself. A flat layout keeps the cost of navigating the project proportional to the project's actual size.

## Bad

```
// A 15-class CLI tool split into five Gradle modules "for future growth"
todo-cli/
├── settings.gradle.kts
├── todo-api/
│   ├── build.gradle.kts
│   └── src/main/java/com/example/todo/api/Task.java
├── todo-core/
│   ├── build.gradle.kts
│   └── src/main/java/com/example/todo/core/TaskService.java
├── todo-storage/
│   ├── build.gradle.kts
│   └── src/main/java/com/example/todo/storage/TaskRepository.java
└── todo-cli/
    ├── build.gradle.kts
    └── src/main/java/com/example/todo/cli/Main.java
// Every class touch requires figuring out which of four build files to edit,
// and inter-module dependency wiring for a project nobody else will ever reuse.
```

## Good

```
todo-cli/
├── build.gradle.kts
└── src/
    ├── main/
    │   └── java/com/example/todo/
    │       ├── Main.java
    │       ├── Task.java
    │       ├── TaskService.java
    │       └── TaskRepository.java
    └── test/
        └── java/com/example/todo/TaskServiceTest.java
```

```kotlin
// build.gradle.kts - one module, one build file, minimal ceremony
plugins {
    application
    java
}

application {
    mainClass.set("com.example.todo.Main")
}
```

## When to Split Later

```
// Split into modules only when a real forcing function appears:
// - a second team needs to own and deploy part of the codebase independently
// - build times become painful and incremental compilation would help
// - a package needs to be published and reused as its own library
// Until one of those is true, the flat layout is the right size.
```

## See Also

- [`proj-multi-module-build`](proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
- [`proj-maven-gradle-standard-layout`](proj-maven-gradle-standard-layout.md) - Follow the standard Maven/Gradle source layout
- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by technical layer
- [`perf-avoid-premature-optimization`](perf-avoid-premature-optimization.md) - Don't optimize before profiling
