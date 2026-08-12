# doc-package-info

> Document packages with `package-info.java`

## Why It Matters

`package-info.java` is the only place to attach Javadoc to a package as a whole, and it is also where package-level annotations (like a JSpecify `@NullMarked` or JSR-305 `@ParametersAreNonnullByDefault`) must live - without it, a package has no documented purpose visible in generated Javadoc, and readers must infer its role by opening every class inside it. Large codebases without `package-info.java` accumulate packages whose intended boundaries and responsibilities are only known by tribal knowledge.

## Bad

```java
// com/opswat/filescanner/detection/DetectionEngine.java
package com.opswat.filescanner.detection;

// No package-info.java exists anywhere in this package.
// A new engineer opening this package has no summary of what
// "detection" is responsible for versus neighboring packages
// like "scanning" or "remediation".
public class DetectionEngine {
    // ...
}
```

## Good

```java
/**
 * Malware detection engines and signature matching.
 *
 * <p>Classes in this package take a scanned file's extracted content
 * (see {@code com.opswat.filescanner.scanning}) and evaluate it against
 * signature and heuristic rule sets to produce a {@link
 * com.opswat.filescanner.detection.DetectionResult}.
 *
 * <p>This package does not perform remediation; see
 * {@code com.opswat.filescanner.remediation} for quarantine and
 * cleanup actions.
 *
 * @since 2.3
 */
package com.opswat.filescanner.detection;
```

```java
// com/opswat/filescanner/detection/DetectionEngine.java
package com.opswat.filescanner.detection;

public class DetectionEngine {
    // ...
}
```

## Package-Level Annotations

`package-info.java` is also the standard location for annotations that apply to every type in the package, such as null-safety defaults:

```java
/**
 * Malware detection engines and signature matching.
 */
@org.jspecify.annotations.NullMarked
package com.opswat.filescanner.detection;
```

Every package intended for public or cross-team consumption should have a `package-info.java`; purely internal, single-author utility packages in a small application are a reasonable place to skip it if the team agrees the cost outweighs the benefit.

## See Also

- [`name-packages-lowercase`](name-packages-lowercase.md) - Use all-lowercase, reverse-domain package names
- [`doc-module-info-documentation`](doc-module-info-documentation.md) - Document JPMS modules in module-info.java
- [`null-jspecify-nullmarked`](null-jspecify-nullmarked.md) - Use JSpecify @NullMarked for null-safety defaults
- [`doc-readme-module-level`](doc-readme-module-level.md) - Maintain a README per module with purpose and usage
