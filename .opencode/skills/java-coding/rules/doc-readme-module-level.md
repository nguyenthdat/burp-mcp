# doc-readme-module-level

> Maintain a README per module with purpose and usage

## Why It Matters

Javadoc documents individual types and methods well, but it rarely answers the first questions a new contributor or consumer has about a module: what problem does this solve, how does it fit with sibling modules, and what's the minimal example to get started. A per-module README fills that gap and is the first thing rendered on GitHub, in an IDE's project tree, or by internal developer portals, so its absence is often the reason a well-built module goes unused or gets reimplemented elsewhere.

## Bad

```
filescanner-detection/
  src/main/java/...
  pom.xml
  # No README - a new team member has to read Javadoc across a dozen
  # classes, or ask in Slack, just to learn this module exists and
  # what it is for.
```

## Good

```
filescanner-detection/
  README.md
  src/main/java/...
  pom.xml
```

```markdown
# filescanner-detection

Malware detection engines and signature matching for the File Scanner platform.

## Purpose

Evaluates scanned file content against signature and heuristic rule sets,
producing a `DetectionResult`. This module does not scan files itself
(see `filescanner-scanning`) and does not perform remediation
(see `filescanner-remediation`).

## Usage

```java
DetectionEngine engine = DetectionEngine.builder()
        .withSignatureSet(SignatureSet.loadDefault())
        .build();

DetectionResult result = engine.evaluate(scannedContent);
if (result.isMalicious()) {
    // ...
}
```

## Module Layout

- `com.opswat.filescanner.detection.api` - public entry points (`DetectionEngine`)
- `com.opswat.filescanner.detection.signatures` - signature set loading and matching
- `com.opswat.filescanner.detection.internal` - not part of the public API

## Building and Testing

    ./mvnw -pl filescanner-detection test

## See Also

- [filescanner-scanning](../filescanner-scanning/README.md)
- [filescanner-remediation](../filescanner-remediation/README.md)
```

## Keeping the README in Sync

A README that drifts from the code is worse than none - link it from `package-info.java` where practical, and treat README updates as part of the definition-of-done for any change to a module's public API, not an afterthought:

```java
/**
 * Malware detection engines and signature matching.
 *
 * <p>See the module README for an overview and usage examples:
 * {@code filescanner-detection/README.md}.
 */
package com.opswat.filescanner.detection;
```

## See Also

- [`doc-package-info`](doc-package-info.md) - Document packages with package-info.java
- [`doc-module-info-documentation`](doc-module-info-documentation.md) - Document JPMS modules in module-info.java
- [`doc-javadoc-code-samples`](doc-javadoc-code-samples.md) - Include runnable code samples in Javadoc
- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by layer
