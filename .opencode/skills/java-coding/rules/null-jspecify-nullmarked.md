# null-jspecify-nullmarked

> Adopt `@NullMarked` (JSpecify) at the package level

## Why It Matters

Annotating every single parameter and return type with `@NonNull` is so noisy that teams give up on nullability annotations entirely. `@NullMarked` flips the default for an entire package to non-null, so you only ever write `@Nullable` where it actually matters, and tools like NullAway or IntelliJ can enforce the contract across the whole codebase with a fraction of the annotation burden.

## Bad

```java
// Every type needs an explicit annotation, or nullability is simply unknown
package com.example.billing;

public class InvoiceService {

    public Invoice createInvoice(@org.jspecify.annotations.NonNull Customer customer,
                                  @org.jspecify.annotations.NonNull List<@org.jspecify.annotations.NonNull LineItem> items) {
        // ... noisy, and easy to forget one annotation, silently losing the guarantee
        return new Invoice(customer, items);
    }
}
```

## Good

```java
// package-info.java in com/example/billing/
@NullMarked
package com.example.billing;

import org.jspecify.annotations.NullMarked;
```

```java
package com.example.billing;

import org.jspecify.annotations.Nullable;

public class InvoiceService {

    // customer, items, and the return type are all non-null by default now
    public Invoice createInvoice(Customer customer, List<LineItem> items) {
        return new Invoice(customer, items);
    }

    // Only the genuinely nullable case needs an annotation
    public @Nullable Invoice findDraft(String customerId) {
        return draftStore.get(customerId);
    }
}
```

## Rolling Out Incrementally

```java
// Mark new/modernized packages @NullMarked first, then expand outward.
// Legacy packages without the annotation stay in "unspecified" nullness,
// so NullAway won't false-positive on code you haven't audited yet.
@NullMarked
package com.example.billing.invoicing;
```

## See Also

- [`null-nullable-annotation`](null-nullable-annotation.md) - Annotate nullability with `@Nullable`/`@NonNull`
- [`lint-nullaway-annotation-checking`](lint-nullaway-annotation-checking.md) - Enforce nullability with NullAway in CI
- [`doc-package-info`](doc-package-info.md) - Document packages with `package-info.java`
- [`proj-package-by-feature`](proj-package-by-feature.md) - Organize packages by feature
