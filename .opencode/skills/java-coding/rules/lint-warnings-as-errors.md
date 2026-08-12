# lint-warnings-as-errors

> Treat compiler warnings as errors in CI

## Why It Matters

Warnings that never fail a build get ignored, and once ignored they accumulate until real problems are buried in noise nobody reads. Turning `-Werror` (or the Java `-Xlint:all -Werror` equivalent) on in CI forces every warning to be fixed or explicitly suppressed with a reason the moment it appears, keeping the signal-to-noise ratio high forever.

## Bad

```groovy
// build.gradle - warnings print to the console and are never addressed
tasks.withType(JavaCompile).configureEach {
    options.compilerArgs << '-Xlint:all'
    // No -Werror: hundreds of unchecked/deprecation warnings accumulate,
    // and the one warning that matters is invisible in the noise.
}
```

```
warning: [unchecked] unchecked conversion
warning: [deprecation] parseInt(String,int) in Integer has been deprecated
warning: [rawtypes] found raw type: List
... 214 more warnings, build succeeds anyway ...
```

## Good

```groovy
// build.gradle
tasks.withType(JavaCompile).configureEach {
    options.compilerArgs << '-Xlint:all' << '-Werror'
    options.deprecation = true
}
```

```xml
<!-- Maven equivalent -->
<plugin>
  <groupId>org.apache.maven.plugins</groupId>
  <artifactId>maven-compiler-plugin</artifactId>
  <version>3.13.0</version>
  <configuration>
    <compilerArgs>
      <arg>-Xlint:all</arg>
      <arg>-Werror</arg>
    </compilerArgs>
    <showWarnings>true</showWarnings>
  </configuration>
</plugin>
```

## Rolling It Out on a Legacy Codebase

```groovy
// Step 1: enable per-category, not all at once, to avoid a wall of failures
tasks.withType(JavaCompile).configureEach {
    options.compilerArgs << '-Xlint:deprecation,unchecked' << '-Werror'
}

// Step 2: once clean, widen the net
tasks.withType(JavaCompile).configureEach {
    options.compilerArgs << '-Xlint:all' << '-Werror'
}
```

## CI Wiring

```yaml
# .github/workflows/ci.yml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: '21'
      - name: Compile (warnings fail the build)
        run: ./gradlew compileJava compileTestJava
```

## Escape Hatch for Genuinely Unavoidable Warnings

```java
@SuppressWarnings("deprecation")
// WHY: no replacement API exists yet for this legacy SDK call; tracked in JIRA-991.
Date parsed = DateParser.parse(input);
```

## See Also

- [`lint-suppress-with-justification`](lint-suppress-with-justification.md) - How to opt out of a specific warning without disabling the whole gate
- [`lint-checkstyle-google-style`](lint-checkstyle-google-style.md) - Style-layer equivalent of failing the build on violations
- [`lint-error-prone-compiler-plugin`](lint-error-prone-compiler-plugin.md) - Adds an additional, stricter layer of compile-time checks
