# lint-checkstyle-google-style

> Enforce a Checkstyle ruleset (Google/Sun style) in CI

## Why It Matters

Without an enforced style ruleset, every pull request turns into a bikeshed over braces, import order, and line length, and reviewers waste time on cosmetics instead of logic. Checkstyle catches these mechanically and consistently before a human ever looks at the diff. Running it in CI (not just as an IDE hint) guarantees the rule is actually followed, not just suggested.

## Bad

```xml
<!-- pom.xml - no Checkstyle plugin configured at all -->
<build>
  <plugins>
    <plugin>
      <groupId>org.apache.maven.plugins</groupId>
      <artifactId>maven-compiler-plugin</artifactId>
      <version>3.13.0</version>
    </plugin>
    <!-- No style enforcement: every dev formats differently -->
  </plugins>
</build>
```

```java
// Inconsistent style slips into main because nothing blocks it
public class orderService{   // wrong case, no space before brace
    public void  processOrder(Order o){
      if(o==null) throw new IllegalArgumentException("null order");
        int    x=1;
    return;
    }
}
```

## Good

```xml
<!-- pom.xml -->
<build>
  <plugins>
    <plugin>
      <groupId>org.apache.maven.plugins</groupId>
      <artifactId>maven-checkstyle-plugin</artifactId>
      <version>3.4.0</version>
      <configuration>
        <configLocation>google_checks.xml</configLocation>
        <consoleOutput>true</consoleOutput>
        <failsOnError>true</failsOnError>
        <violationSeverity>warning</violationSeverity>
      </configuration>
      <executions>
        <execution>
          <id>checkstyle-check</id>
          <phase>verify</phase>
          <goals>
            <goal>check</goal>
          </goals>
        </execution>
      </executions>
    </plugin>
  </plugins>
</build>
```

```java
public class OrderService {

  public void processOrder(Order order) {
    if (order == null) {
      throw new IllegalArgumentException("order must not be null");
    }
    int quantity = 1;
  }
}
```

## Gradle Equivalent

```groovy
plugins {
    id 'checkstyle'
}

checkstyle {
    toolVersion = '10.17.0'
    configFile = file("${rootDir}/config/checkstyle/google_checks.xml")
    maxWarnings = 0
    ignoreFailures = false
}

tasks.named('check') {
    dependsOn 'checkstyleMain', 'checkstyleTest'
}
```

## Custom Ruleset Snippet

```xml
<!-- config/checkstyle/checkstyle.xml - tailored subset -->
<module name="Checker">
  <property name="charset" value="UTF-8"/>
  <module name="TreeWalker">
    <module name="UnusedImports"/>
    <module name="RedundantImport"/>
    <module name="LineLength">
      <property name="max" value="120"/>
    </module>
    <module name="MethodLength">
      <property name="max" value="80"/>
    </module>
  </module>
</module>
```

## See Also

- [`lint-pmd-rulesets`](lint-pmd-rulesets.md) - Complementary static analysis for bug patterns, not just style
- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - Make style violations fail the build
- [`lint-editorconfig-formatting`](lint-editorconfig-formatting.md) - Auto-fix formatting instead of just reporting it
- [`lint-suppress-with-justification`](lint-suppress-with-justification.md) - How to handle unavoidable violations
