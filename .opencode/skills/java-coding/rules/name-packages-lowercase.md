# name-packages-lowercase

> Use all-lowercase, reverse-domain package names

## Why It Matters

Package names are all-lowercase by convention because the JVM's file system mapping and case-sensitivity rules differ across platforms (macOS/Windows are case-insensitive by default, Linux is not), so mixed-case packages risk classpath collisions that only surface on certain machines or CI runners. The reverse-domain prefix (`com.company.project`) guarantees global uniqueness across dependencies, avoiding silent class shadowing when two libraries pick the same short package name.

## Bad

```java
package com.OpsWat.FileScanner.Utils;  // mixed case, breaks convention

public class HashCalculator {
    // ...
}
```

```java
package fileScanner;  // no reverse-domain prefix, collision-prone

public class HashCalculator {
    // ...
}
```

## Good

```java
package com.opswat.filescanner.util;

public class HashCalculator {
    // ...
}
```

## Multi-Word Segments

Java package segments cannot contain hyphens or underscores as a matter of convention (they are legal in identifiers but discouraged), so multi-word segments are simply concatenated in lowercase rather than delimited.

```java
// Preferred: concatenated lowercase
package com.opswat.filescanner.malwaredetection;

// Avoid: underscores or camelCase segments
package com.opswat.file_scanner.malwareDetection;
```

When a segment would start with a digit or match a reserved keyword, prepend an underscore only as a last resort (e.g., `com.example._2024.report`), since numeric-leading or keyword segments are illegal otherwise.

## See Also

- [`name-classes-pascal`](name-classes-pascal.md) - Use PascalCase for classes, interfaces, enums, records
- [`doc-package-info`](doc-package-info.md) - Document packages with package-info.java
- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature, not by layer
