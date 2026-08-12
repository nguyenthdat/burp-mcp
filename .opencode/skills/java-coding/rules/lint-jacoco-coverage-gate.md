# lint-jacoco-coverage-gate

> Enforce a JaCoCo coverage gate in CI

## Why It Matters

Coverage reports that nobody enforces trend downward as deadlines slip, and by the time someone notices, entire modules are untested. A JaCoCo gate that fails the build below a threshold keeps coverage from silently regressing, and per-package/per-class rules let you hold critical modules (payments, auth) to a higher bar than boilerplate DTOs.

## Bad

```groovy
// build.gradle - report generated, never checked
plugins {
    id 'jacoco'
}

jacocoTestReport {
    reports {
        xml.required = true
    }
}
// No jacocoTestCoverageVerification task wired to `check`,
// so coverage can drop to 0% without CI noticing.
```

## Good

```groovy
// build.gradle
plugins {
    id 'jacoco'
}

jacoco {
    toolVersion = '0.8.12'
}

jacocoTestReport {
    dependsOn test
    reports {
        xml.required = true
        html.required = true
    }
}

jacocoTestCoverageVerification {
    dependsOn jacocoTestReport
    violationRules {
        rule {
            limit {
                counter = 'LINE'
                minimum = 0.80
            }
        }
        rule {
            element = 'CLASS'
            includes = ['com.example.payments.*']
            limit {
                counter = 'BRANCH'
                minimum = 0.90
            }
        }
    }
}

tasks.named('check') {
    dependsOn jacocoTestCoverageVerification
}
```

## Maven Equivalent

```xml
<plugin>
  <groupId>org.jacoco</groupId>
  <artifactId>jacoco-maven-plugin</artifactId>
  <version>0.8.12</version>
  <executions>
    <execution>
      <goals><goal>prepare-agent</goal></goals>
    </execution>
    <execution>
      <id>report</id>
      <phase>test</phase>
      <goals><goal>report</goal></goals>
    </execution>
    <execution>
      <id>check</id>
      <goals><goal>check</goal></goals>
      <configuration>
        <rules>
          <rule>
            <element>BUNDLE</element>
            <limits>
              <limit>
                <counter>LINE</counter>
                <minimum>0.80</minimum>
              </limit>
            </limits>
          </rule>
        </rules>
      </configuration>
    </execution>
  </executions>
</plugin>
```

## Excluding Generated Code from the Gate

```groovy
jacocoTestReport {
    afterEvaluate {
        classDirectories.setFrom(files(classDirectories.files.collect {
            fileTree(dir: it, exclude: [
                '**/*_MapperImpl.class',   // MapStruct generated
                '**/dto/**',
                '**/*$Builder.class'
            ])
        }))
    }
}
```

## See Also

- [`lint-dependency-vulnerability-scan`](lint-dependency-vulnerability-scan.md) - Another CI-enforced quality gate with a numeric threshold
- [`test-one-concept-per-test`](test-one-concept-per-test.md) - Coverage percentage is necessary but not sufficient; pair with meaningful, focused assertions
- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - The general "fail the build" enforcement pattern
