# anti-catch-and-ignore

> Don't catch and silently ignore exceptions

## Why It Matters

An empty catch block turns a real failure into a phantom bug: the operation didn't succeed, but nothing logs it, nothing alerts on it, and the caller proceeds as if everything worked. These are among the hardest bugs to diagnose because by the time symptoms appear (corrupted state, missing data), the original exception and its stack trace are long gone.

## Bad

```java
public void syncInventory(String warehouseId) {
  try {
    inventoryClient.sync(warehouseId);
  } catch (IOException e) {
    // Swallowed - the sync silently failed, inventory is now stale,
    // and nobody will know until a customer orders an out-of-stock item
  }
}

public int parseQuantity(String raw) {
  try {
    return Integer.parseInt(raw);
  } catch (NumberFormatException e) {
  }
  return 0; // Indistinguishable from a legitimate "0" input
}
```

## Good

```java
public void syncInventory(String warehouseId) throws InventorySyncException {
  try {
    inventoryClient.sync(warehouseId);
  } catch (IOException e) {
    throw new InventorySyncException(
        "failed to sync inventory for warehouse " + warehouseId, e);
  }
}

public OptionalInt parseQuantity(String raw) {
  try {
    return OptionalInt.of(Integer.parseInt(raw));
  } catch (NumberFormatException e) {
    log.warn("invalid quantity '{}', treating as absent", raw, e);
    return OptionalInt.empty();
  }
}
```

## When an Empty-Looking Catch Is Actually Fine

```java
// Explicitly documented, deliberate best-effort cleanup - the comment
// makes clear this was a choice, not an oversight.
try {
  tempFile.delete();
} catch (SecurityException e) {
  // Ignored: best-effort cleanup of a temp file; leaving it behind
  // is harmless and the OS will reclaim it eventually.
}
```

## Static Analysis Catches This

```xml
<!-- PMD flags this automatically -->
<rule ref="category/java/errorprone.xml/EmptyCatchBlock"/>
<!-- SpotBugs: DE_MIGHT_IGNORE / REC_CATCH_EXCEPTION also flag related patterns -->
```

## See Also

- [`err-no-empty-catch`](err-no-empty-catch.md) - The positive rule this anti-pattern violates
- [`err-exception-chaining`](err-exception-chaining.md) - Preserve the original cause instead of dropping it
- [`anti-excessive-checked-exceptions`](anti-excessive-checked-exceptions.md) - A common root cause that pushes developers toward swallowing exceptions
- [`lint-pmd-rulesets`](lint-pmd-rulesets.md) - Static analysis that flags empty catch blocks automatically
