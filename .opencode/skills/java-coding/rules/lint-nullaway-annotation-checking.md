# lint-nullaway-annotation-checking

> Use NullAway for compile-time null-safety checks

## Why It Matters

NullAway (built on Error Prone) statically verifies `@Nullable` contracts during compilation, catching most `NullPointerException` risks before the code ever runs, at a fraction of the cost of full formal verification tools. Unlike a runtime null check, a compile failure means the bug never reaches a code review or a production incident in the first place.

## Bad

```groovy
// build.gradle - no null-safety checking at all
plugins {
    id 'java'
}
// Nothing catches this until it NPEs in production:
```

```java
public class UserService {
  @Nullable
  private String cachedName;

  public int getNameLength() {
    return cachedName.length(); // Compiles fine, NPEs at runtime whenever cache is empty
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
    errorprone 'com.uber.nullaway:nullaway:0.11.3'
    compileOnly 'com.google.code.findbugs:jsr305:3.0.2'
}

tasks.withType(JavaCompile).configureEach {
    options.errorprone {
        check('NullAway', net.ltgt.gradle.errorprone.CheckSeverity.ERROR)
        option('NullAway:AnnotatedPackages', 'com.example.myapp')
        option('NullAway:TreatGeneratedAsUnannotated', 'true')
    }
}
```

```java
public class UserService {
  private @Nullable String cachedName;

  public int getNameLength() {
    if (cachedName == null) {
      return 0; // NullAway forces this branch to exist before dereferencing
    }
    return cachedName.length();
  }
}
```

## Maven Equivalent

```xml
<annotationProcessorPaths>
  <path>
    <groupId>com.google.errorprone</groupId>
    <artifactId>error_prone_core</artifactId>
    <version>2.31.0</version>
  </path>
  <path>
    <groupId>com.uber.nullaway</groupId>
    <artifactId>nullaway</artifactId>
    <version>0.11.3</version>
  </path>
</annotationProcessorPaths>
<compilerArgs>
  <arg>-Xplugin:ErrorProne -Xep:NullAway:ERROR -XepOpt:NullAway:AnnotatedPackages=com.example.myapp</arg>
</compilerArgs>
```

## What Gets Flagged

```java
public String formatUser(@Nullable User user) {
  return user.getName(); // NullAway ERROR: dereference of @Nullable parameter
}

public String formatUserFixed(@Nullable User user) {
  return user == null ? "unknown" : user.getName(); // OK
}

// Fields must be initialized or explicitly @Nullable
public class Config {
  private String host; // NullAway ERROR if never assigned in constructor
}
```

## Migrating an Existing Codebase

```groovy
// Roll out incrementally, package by package, instead of a big-bang switch
option('NullAway:AnnotatedPackages', 'com.example.myapp.newmodule')
option('NullAway:UnannotatedSubPackages', 'com.example.myapp.legacy')
```

## See Also

- [`null-jspecify-nullmarked`](null-jspecify-nullmarked.md) - The `@NullMarked` annotation model NullAway increasingly supports
- [`null-nullable-annotation`](null-nullable-annotation.md) - How to annotate fields/params/returns correctly
- [`lint-error-prone-compiler-plugin`](lint-error-prone-compiler-plugin.md) - The compiler framework NullAway plugs into
- [`anti-null-check-cascade`](anti-null-check-cascade.md) - What over-defensive code looks like without this discipline
