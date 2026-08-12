# doc-comment-why-not-what

> Write comments explaining why, not what the code already says

## Why It Matters

Code that restates itself in comments doubles the maintenance surface without adding information - when the code changes and the comment does not, the comment becomes actively misleading, which is worse than no comment at all. Comments earn their keep by capturing information the code cannot express: the reasoning behind a non-obvious choice, a business constraint, a workaround for an external bug, or a warning about a subtle failure mode.

## Bad

```java
public class PriceCalculator {

    public double calculateTotal(double price, int quantity) {
        // multiply price by quantity
        double subtotal = price * quantity;

        // increment retry count by 1
        retryCount++;

        // if quantity is greater than 100
        if (quantity > 100) {
            // apply a 0.05 discount
            subtotal = subtotal * 0.95;
        }

        // return the subtotal
        return subtotal;
    }
}
```

## Good

```java
public class PriceCalculator {

    public double calculateTotal(double price, int quantity) {
        double subtotal = price * quantity;
        retryCount++;

        // Bulk discount required by Sales Ops contract SO-2024-118;
        // threshold and rate must match the signed vendor agreement,
        // not the generic loyalty discount used elsewhere.
        if (quantity > 100) {
            subtotal = subtotal * 0.95;
        }

        return subtotal;
    }
}
```

## When a Comment Is Still Worth Writing

Comments are valuable for non-obvious algorithms, workarounds for library bugs, performance trade-offs, or anything a reader would otherwise have to reverse-engineer from git history:

```java
public int findInsertionPoint(int[] sortedArray, int target) {
    // Binary search rather than a linear scan: this runs per-item during
    // bulk import (see ImportJob), and profiling showed linear scan was
    // 40% of total import time at 200k+ SKUs.
    int low = 0;
    int high = sortedArray.length - 1;
    while (low <= high) {
        int mid = (low + high) >>> 1;  // avoids overflow vs. (low + high) / 2
        if (sortedArray[mid] < target) {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    return low;
}
```

```java
// Workaround for JDK-8274349: Files.copy() with REPLACE_EXISTING can throw
// AccessDeniedException on Windows if the destination is memory-mapped
// elsewhere. Retry once after a short delay before failing.
Files.copy(source, destination, StandardCopyOption.REPLACE_EXISTING);
```

If a comment can be deleted without losing any information a reader needs, delete it; if the logic is genuinely self-explanatory, prefer a well-named method or variable over a comment at all.

## See Also

- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Document all public API with Javadoc
- [`err-exception-message-context`](err-exception-message-context.md) - Include actionable context in exception messages
- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
