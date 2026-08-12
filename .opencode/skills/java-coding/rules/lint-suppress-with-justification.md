# lint-suppress-with-justification

> Require a comment justification alongside any suppression

## Why It Matters

An unexplained `@SuppressWarnings` is a silent bet that the linter is wrong, and six months later nobody remembers whether that bet was safe. Requiring a justification comment (and ideally a ticket reference) turns every suppression into a documented, reviewable decision instead of a permanent blind spot that accumulates over time.

## Bad

```java
@SuppressWarnings("unchecked")
public <T> T convert(Object value) {
  return (T) value; // No explanation: is this actually safe, or was it just to silence a warning?
}

@SuppressWarnings({"unused", "rawtypes"})
public class LegacyAdapter {
  // Two suppressions stacked with zero context - safe to remove? Nobody knows.
}
```

## Good

```java
@SuppressWarnings("unchecked")
// Safe: caller-supplied Class<T> guarantees the runtime type matches T (see registerType()).
public <T> T convert(Object value, Class<T> type) {
  return (T) type.cast(value);
}
```

```java
// PMD/SpotBugs suppressions in the same style
@SuppressFBWarnings(
    value = "EI_EXPOSE_REP",
    justification = "Returned list is intentionally mutable for the builder API; documented in Javadoc.")
public List<String> getItemsForBuilder() {
  return items;
}
```

## Enforcing the Rule with Checkstyle

```xml
<!-- config/checkstyle/checkstyle.xml -->
<module name="SuppressWarningsHolder"/>
<module name="RegexpSinglelineJava">
  <property name="format" value="@SuppressWarnings"/>
  <property name="message"
      value="Suppressions must be followed by a // justification comment on the next line."/>
  <property name="ignoreComments" value="false"/>
</module>
```

## PMD Suppression With Reason

```java
// PMD requires a reason via the standard annotation attribute where supported,
// otherwise document inline immediately above the suppression.
@SuppressWarnings("PMD.CyclomaticComplexity")
// Justification: this parser mirrors a fixed external grammar (RFC 5322);
// splitting it would scatter state across multiple methods. Tracked in JIRA-4821.
public Token parseHeader(String raw) {
  // ...
  return null;
}
```

## Team Convention

```java
// Required format for every suppression in this codebase:
// @SuppressWarnings("<lint>")
// // WHY: <reason it's safe> [<ticket-id if long-lived>]
@SuppressWarnings("unchecked")
// WHY: array creation via reflection is inherently unchecked; bounded by
// the ClassCastException-safe cast on the next line. See TICKET-1123.
private static <T> T[] newArray(Class<T> componentType, int size) {
  return (T[]) java.lang.reflect.Array.newInstance(componentType, size);
}
```

## See Also

- [`lint-warnings-as-errors`](lint-warnings-as-errors.md) - Why suppressions must be deliberate, not a workaround for an over-strict gate
- [`type-safevarargs-heap-pollution`](type-safevarargs-heap-pollution.md) - A common, legitimately-safe use of `@SuppressWarnings("unchecked")`
- [`lint-spotbugs-ci-gate`](lint-spotbugs-ci-gate.md) - Where `@SuppressFBWarnings` justifications matter most
