# lint-editorconfig-formatting

> Enforce formatting via `.editorconfig`/Spotless

## Why It Matters

Manually formatted code drifts between contributors and editors (tabs vs spaces, trailing whitespace, brace placement), and Checkstyle alone only reports violations rather than fixing them. `.editorconfig` gives every editor a shared baseline, and a formatter like Spotless (backed by google-java-format) auto-rewrites files in a pre-commit hook or CI step so formatting is never a discussion topic in review.

## Bad

```
# No .editorconfig in the repo at all.
# Every contributor's IDE uses its own default indentation and line-ending
# settings, so diffs are full of unrelated whitespace churn.
```

```groovy
// build.gradle - no formatter plugin, so nothing normalizes style
plugins {
    id 'java'
}
```

## Good

```ini
# .editorconfig
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.java]
indent_style = space
indent_size = 4
max_line_length = 120

[*.{yml,yaml,json}]
indent_style = space
indent_size = 2
```

```groovy
// build.gradle
plugins {
    id 'com.diffplug.spotless' version '6.25.0'
}

spotless {
    java {
        googleJavaFormat('1.22.0')
        removeUnusedImports()
        trimTrailingWhitespace()
        endWithNewline()
        target 'src/*/java/**/*.java'
    }
}

tasks.named('check') {
    dependsOn 'spotlessCheck'
}
```

## CI Wiring (Fails If Files Aren't Formatted)

```yaml
# .github/workflows/ci.yml
jobs:
  format:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify formatting
        run: ./gradlew spotlessCheck
```

## Auto-Fix Locally Before Committing

```bash
# Rewrites files in place instead of just reporting violations
./gradlew spotlessApply

# Pre-commit hook (.git/hooks/pre-commit or via husky/lefthook)
./gradlew spotlessApply
git add -u
```

## Maven Equivalent

```xml
<plugin>
  <groupId>com.diffplug.spotless</groupId>
  <artifactId>spotless-maven-plugin</artifactId>
  <version>2.43.0</version>
  <configuration>
    <java>
      <googleJavaFormat><version>1.22.0</version></googleJavaFormat>
    </java>
  </configuration>
  <executions>
    <execution>
      <goals><goal>check</goal></goals>
      <phase>verify</phase>
    </execution>
  </executions>
</plugin>
```

## See Also

- [`lint-checkstyle-google-style`](lint-checkstyle-google-style.md) - Reports style violations; Spotless additionally auto-fixes them
- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - `spotlessCheck` failing the build follows the same enforcement pattern
