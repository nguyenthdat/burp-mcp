# lint-spotbugs-ci-gate

> Run SpotBugs as a required CI gate

## Why It Matters

SpotBugs finds real bug patterns (null dereferences, resource leaks, incorrect equals/hashCode, bad synchronization) by analyzing bytecode, catching classes of defects that compile cleanly but fail at runtime. If it only runs locally or advisory, regressions merge anyway; making it a required, blocking CI check is the only way it actually prevents bugs from shipping.

## Bad

```groovy
// build.gradle - SpotBugs configured but never enforced
plugins {
    id 'com.github.spotbugs' version '6.0.15'
}

spotbugsMain {
    ignoreFailures = true   // Findings are printed then ignored
    reportsDir = file("$buildDir/reports/spotbugs")
}
// No CI step ever fails the build, so bugs pile up silently
```

## Good

```groovy
// build.gradle
plugins {
    id 'com.github.spotbugs' version '6.0.15'
}

spotbugs {
    toolVersion = '4.8.6'
    effort = 'max'
    reportLevel = 'medium'
}

spotbugsMain {
    ignoreFailures = false
    reports {
        html.required = true
        xml.required = true
    }
}

tasks.named('check') {
    dependsOn 'spotbugsMain', 'spotbugsTest'
}
```

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
      - name: Build and verify (fails on SpotBugs findings)
        run: ./gradlew check spotbugsMain
```

## Maven Equivalent

```xml
<plugin>
  <groupId>com.github.spotbugs</groupId>
  <artifactId>spotbugs-maven-plugin</artifactId>
  <version>4.8.6.1</version>
  <configuration>
    <effort>Max</effort>
    <threshold>Medium</threshold>
    <failOnError>true</failOnError>
  </configuration>
  <executions>
    <execution>
      <id>spotbugs-check</id>
      <phase>verify</phase>
      <goals>
        <goal>check</goal>
      </goals>
    </execution>
  </executions>
</plugin>
```

## Typical Findings It Catches

```java
// EI_EXPOSE_REP: exposing internal mutable state
public List<String> getItems() {
  return items; // SpotBugs flags: caller can mutate internal list
}

// NP_NULL_ON_SOME_PATH: possible null dereference
String name = user.getName();
if (condition) {
  name = null;
}
System.out.println(name.length()); // Flagged: NPE risk

// DM_DEFAULT_ENCODING: platform-dependent charset
byte[] bytes = text.getBytes(); // Flagged: use getBytes(StandardCharsets.UTF_8)
```

## See Also

- [`lint-error-prone-compiler-plugin`](lint-error-prone-compiler-plugin.md) - Catches similar bug patterns at compile time instead of post-compile
- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - The general principle of failing builds on findings
- [`lint-suppress-with-justification`](lint-suppress-with-justification.md) - How to handle unavoidable findings
