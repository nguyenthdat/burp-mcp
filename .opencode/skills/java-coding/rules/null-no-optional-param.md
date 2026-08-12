# null-no-optional-param

> Never accept `Optional<T>` as a method parameter

## Why It Matters

`Optional` parameters force every caller to wrap a value just to call the method, and they still don't stop someone from passing `Optional.ofNullable(null)` or an actual `null` for the `Optional` itself — you end up defending against the exact problem `Optional` was supposed to remove. Overloads, a nullable parameter, or a builder communicate optionality more clearly and with less ceremony.

## Bad

```java
public class ReportService {

    // Every caller must wrap arguments in Optional
    public Report generate(String title, Optional<LocalDate> asOf, Optional<String> author) {
        LocalDate date = asOf.orElse(LocalDate.now());
        String reportAuthor = author.orElse("system");
        return new Report(title, date, reportAuthor);
    }
}

// Awkward call sites
service.generate("Q3 Summary", Optional.of(LocalDate.of(2026, 9, 30)), Optional.empty());
service.generate("Q3 Summary", Optional.empty(), Optional.empty());
```

## Good

```java
public class ReportService {

    public Report generate(String title) {
        return generate(title, LocalDate.now(), "system");
    }

    public Report generate(String title, LocalDate asOf) {
        return generate(title, asOf, "system");
    }

    public Report generate(String title, LocalDate asOf, String author) {
        return new Report(title, asOf, author);
    }
}

// Clean call sites, no wrapping required
service.generate("Q3 Summary");
service.generate("Q3 Summary", LocalDate.of(2026, 9, 30));
```

## Builder Alternative For Many Optional Fields

```java
public class ReportRequest {

    private final String title;
    private LocalDate asOf = LocalDate.now();
    private String author = "system";

    private ReportRequest(String title) {
        this.title = title;
    }

    public static ReportRequest forTitle(String title) {
        return new ReportRequest(title);
    }

    public ReportRequest asOf(LocalDate date) {
        this.asOf = date;
        return this;
    }

    public ReportRequest author(String author) {
        this.author = author;
        return this;
    }
}

// service.generate(ReportRequest.forTitle("Q3 Summary").author("alice"));
```

## See Also

- [`null-optional-return-type`](null-optional-return-type.md) - Use `Optional<T>` for return types only
- [`null-no-optional-field`](null-no-optional-field.md) - Never declare a field of type `Optional<T>`
- [`api-builder-complex-construction`](api-builder-complex-construction.md) - Use a builder for complex construction
- [`api-avoid-telescoping-constructors`](api-avoid-telescoping-constructors.md) - Avoid telescoping constructors
