# proj-maven-gradle-standard-layout

> Follow the standard Maven/Gradle source layout

## Why It Matters

Maven and Gradle both default to the same "standard directory layout" convention (`src/main/java`, `src/test/java`, `src/main/resources`), and every IDE, plugin, and CI tool assumes it unless told otherwise. Deviating from it forces custom `sourceSets` or `<build>` configuration in every project, confuses new contributors, and breaks tooling that scans for source roots by convention rather than configuration.

## Bad

```
my-app/
├── code/
│   ├── com/example/app/Main.java        // Non-standard source root
│   └── com/example/app/OrderService.java
├── tests/
│   └── com/example/app/OrderServiceTest.java  // Non-standard test root
├── config/
│   └── application.properties            // Resources mixed with config, no clear root
└── build.gradle.kts
```

```kotlin
// build.gradle.kts - forced to redeclare source sets to work around the layout
sourceSets {
    main {
        java.setSrcDirs(listOf("code"))
        resources.setSrcDirs(listOf("config"))
    }
    test {
        java.setSrcDirs(listOf("tests"))
    }
}
```

## Good

```
my-app/
├── src/
│   ├── main/
│   │   ├── java/
│   │   │   └── com/example/app/
│   │   │       ├── Main.java
│   │   │       └── OrderService.java
│   │   └── resources/
│   │       └── application.properties
│   └── test/
│       ├── java/
│       │   └── com/example/app/OrderServiceTest.java
│       └── resources/
│           └── application-test.properties
├── build.gradle.kts
└── settings.gradle.kts
```

```kotlin
// build.gradle.kts - no custom sourceSets block needed at all
plugins {
    java
}
```

## Maven Equivalent

```xml
<!-- pom.xml relies on the same src/main/java, src/test/java convention
     by default, so <build><sourceDirectory> is rarely needed. -->
<project>
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>my-app</artifactId>
    <version>1.0.0</version>
</project>
```

## See Also

- [`proj-resources-separation`](proj-resources-separation.md) - Separate resources from source under standard directories
- [`proj-multi-module-build`](proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by technical layer
