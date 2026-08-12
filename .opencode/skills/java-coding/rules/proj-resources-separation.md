# proj-resources-separation

> Separate resources from source under standard directories

## Why It Matters

Mixing configuration files, templates, and static assets directly into `src/main/java` alongside `.java` source confuses build tools that treat that directory as compile input, forces manual classpath configuration, and makes it hard to tell at a glance what is code versus data. Placing non-code assets under `src/main/resources` (and test-only assets under `src/test/resources`) lets Maven/Gradle package them onto the classpath automatically, keeps environment-specific config out of compiled `.class` files, and makes test fixtures clearly scoped to tests only.

## Bad

```
src/main/java/com/example/app/
├── OrderService.java
├── application.yml              // Config file sitting inside the Java source tree
├── email-template.html          // Template mixed in with source
└── logo.png                     // Static asset mixed in with source
```

```java
// Forced to load it via a fragile relative path or custom classpath entry
// because it isn't in a resources root the build recognizes by convention.
InputStream in = new FileInputStream("src/main/java/com/example/app/email-template.html");
```

## Good

```
src/
├── main/
│   ├── java/
│   │   └── com/example/app/OrderService.java
│   └── resources/
│       ├── application.yml
│       ├── templates/email-template.html
│       └── static/logo.png
└── test/
    ├── java/
    │   └── com/example/app/OrderServiceTest.java
    └── resources/
        ├── application-test.yml
        └── fixtures/sample-order.json
```

```java
// Resources on the classpath load reliably regardless of working directory
try (InputStream in = getClass().getResourceAsStream("/templates/email-template.html")) {
    String template = new String(in.readAllBytes(), StandardCharsets.UTF_8);
}
```

```java
// Test-only fixtures never leak into the production JAR because they live
// under src/test/resources, which Maven/Gradle exclude from the main artifact.
Path fixture = Path.of(getClass().getResource("/fixtures/sample-order.json").toURI());
```

## See Also

- [`proj-maven-gradle-standard-layout`](proj-maven-gradle-standard-layout.md) - Follow the standard Maven/Gradle source layout
- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by technical layer
- [`proj-multi-module-build`](proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
