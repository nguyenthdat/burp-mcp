# lint-error-prone-compiler-plugin

> Compile with the Error Prone plugin

## Why It Matters

Error Prone hooks into javac itself, so it catches common Java mistakes (bad equals overrides, mismatched printf arguments, unused return values from immutable-returning methods) at the exact moment the code is compiled, with zero extra pipeline steps. Because it runs as part of `javac`, there is no way to accidentally skip it the way a separate lint step can be skipped.

## Bad

```groovy
// build.gradle - compiling with plain javac, no Error Prone
plugins {
    id 'java'
}
// Bugs like this compile without warning:
```

```java
public class Money {
  private final BigDecimal amount;

  Money(BigDecimal amount) {
    this.amount = amount;
  }

  @Override
  public boolean equals(Object o) {
    // Comparing BigDecimal with == and ignoring scale differences
    return o instanceof Money && ((Money) o).amount == this.amount;
  }
}
```

## Good

```groovy
// build.gradle
plugins {
    id 'java'
    id 'net.ltgt.errorprone' version '4.1.0'
}

dependencies {
    errorprone 'com.google.errorprone:error_prone_core:2.31.0'
}

tasks.withType(JavaCompile).configureEach {
    options.errorprone {
        allErrorsAsWarnings = false
        error('BadComparable', 'EqualsHashCode', 'ArrayEquals')
    }
}
```

```java
public class Money {
  private final BigDecimal amount;

  Money(BigDecimal amount) {
    this.amount = amount;
  }

  @Override
  public boolean equals(Object o) {
    if (!(o instanceof Money other)) {
      return false;
    }
    return amount.compareTo(other.amount) == 0; // Error Prone flags == on BigDecimal
  }

  @Override
  public int hashCode() {
    return amount.stripTrailingZeros().hashCode();
  }
}
```

## Maven Equivalent

```xml
<plugin>
  <groupId>org.apache.maven.plugins</groupId>
  <artifactId>maven-compiler-plugin</artifactId>
  <version>3.13.0</version>
  <configuration>
    <compilerArgs>
      <arg>-XDcompilePolicy=simple</arg>
      <arg>--should-stop=ifError=FLOW</arg>
      <arg>-Xplugin:ErrorProne -Xep:UnusedReturnValue:ERROR</arg>
    </compilerArgs>
    <annotationProcessorPaths>
      <path>
        <groupId>com.google.errorprone</groupId>
        <artifactId>error_prone_core</artifactId>
        <version>2.31.0</version>
      </path>
    </annotationProcessorPaths>
  </configuration>
</plugin>
```

## Bugs It Routinely Catches

```java
// MissingOverride, ReferenceEquality, StringSplitter, DoubleBraceInitialization
if (status == Status.ACTIVE) { }        // Flags: use .equals() for enums? Actually fine for enums,
                                         // but flags reference equality on boxed types like Integer/String

Integer a = 1000;
Integer b = 1000;
if (a == b) { }                         // ReferenceEquality: flagged, use .equals()

String[] parts = "a,b,c".split(",", -1); // StringSplitter: flags fragile split without explicit limit reasoning
```

## See Also

- [`lint-nullaway-annotation-checking`](lint-nullaway-annotation-checking.md) - Built on top of Error Prone for null-safety checks
- [`lint-spotbugs-ci-gate`](lint-spotbugs-ci-gate.md) - Post-compile bytecode analysis as a complementary layer
- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - Make Error Prone findings fail the build
