# lint-pmd-rulesets

> Run PMD rulesets for additional static analysis

## Why It Matters

PMD analyzes source (not bytecode) and catches a different class of issues than Checkstyle or SpotBugs: dead code, overly complex methods, empty catch blocks, and copy-paste duplication via CPD. Running a curated ruleset - not the noisy "everything" default - keeps signal high so developers don't tune the tool out.

## Bad

```xml
<!-- No PMD at all, or the default "quickstart" ruleset with hundreds
     of stylistic rules that nobody triages, so the report is ignored -->
<plugin>
  <groupId>org.apache.maven.plugins</groupId>
  <artifactId>maven-pmd-plugin</artifactId>
  <version>3.24.0</version>
  <!-- no <rulesets> -> uses defaults, produces thousands of low-value warnings -->
</plugin>
```

## Good

```xml
<!-- pom.xml -->
<plugin>
  <groupId>org.apache.maven.plugins</groupId>
  <artifactId>maven-pmd-plugin</artifactId>
  <version>3.24.0</version>
  <configuration>
    <rulesets>
      <ruleset>config/pmd/custom-ruleset.xml</ruleset>
    </rulesets>
    <printFailingErrors>true</printFailingErrors>
    <failOnViolation>true</failOnViolation>
    <targetJdk>21</targetJdk>
  </configuration>
  <executions>
    <execution>
      <phase>verify</phase>
      <goals>
        <goal>check</goal>
        <goal>cpd-check</goal>
      </goals>
    </execution>
  </executions>
</plugin>
```

```xml
<!-- config/pmd/custom-ruleset.xml -->
<?xml version="1.0"?>
<ruleset name="Custom Rules"
    xmlns="http://pmd.sourceforge.net/ruleset/2.0.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://pmd.sourceforge.net/ruleset/2.0.0 https://pmd.sourceforge.io/ruleset_2_0_0.xsd">

  <description>Curated PMD rules for the team</description>

  <rule ref="category/java/errorprone.xml">
    <exclude name="BeanMembersShouldSerialize"/>
  </rule>
  <rule ref="category/java/bestpractices.xml/UnusedLocalVariable"/>
  <rule ref="category/java/bestpractices.xml/AvoidReassigningLoopVariables"/>
  <rule ref="category/java/design.xml/CyclomaticComplexity">
    <properties>
      <property name="classReportLevel" value="80"/>
      <property name="methodReportLevel" value="10"/>
    </properties>
  </rule>
  <rule ref="category/java/errorprone.xml/EmptyCatchBlock"/>
</ruleset>
```

## Gradle Equivalent

```groovy
plugins {
    id 'pmd'
}

pmd {
    toolVersion = '7.3.0'
    ruleSetFiles = files("${rootDir}/config/pmd/custom-ruleset.xml")
    ruleSets = []
    ignoreFailures = false
}
```

## Findings It Routinely Catches

```java
// CyclomaticComplexity: method doing too much
void handleRequest(Request r) {
  if (r.isA()) { /* ... */ } else if (r.isB()) { /* ... */ }
  else if (r.isC()) { /* ... */ } else if (r.isD()) { /* ... */ }
  // 12 branches - flagged, extract strategy objects
}

// EmptyCatchBlock
try {
  risky();
} catch (IOException e) {
  // flagged: silent failure, see anti-catch-and-ignore
}

// UnusedLocalVariable, UselessParentheses, AvoidReassigningLoopVariables
```

## See Also

- [`lint-checkstyle-google-style`](lint-checkstyle-google-style.md) - Style enforcement; PMD focuses on structure and bug patterns instead
- [`lint-spotbugs-ci-gate`](lint-spotbugs-ci-gate.md) - Bytecode-level bug detection as a complement
- [`anti-catch-and-ignore`](anti-catch-and-ignore.md) - The exact pattern PMD's EmptyCatchBlock rule flags
- [`anti-god-class`](anti-god-class.md) - Related to PMD's design/complexity rules
