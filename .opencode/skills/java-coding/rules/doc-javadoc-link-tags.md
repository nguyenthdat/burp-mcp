# doc-javadoc-link-tags

> Use `{@link}`/`{@code}` to cross-reference types and code

## Why It Matters

`{@link}` turns a plain-text type or member name into a clickable cross-reference in generated Javadoc and in most IDEs' hover tooltips, letting readers jump straight to the referenced API instead of searching for it manually. `{@code}` renders inline code (identifiers, literals, snippets) in a monospace font and, critically, suppresses HTML interpretation of special characters like `<` and `>`, which otherwise get silently swallowed or misrendered by the Javadoc HTML renderer.

## Bad

```java
/**
 * Converts the given value using the ValueConverter class. Pass null
 * to use the default converter. Returns a List<String> of results, or
 * an empty list if input is empty.
 */
public List<String> convert(Object value, ValueConverter converter) {
    // "ValueConverter" is plain text - not a clickable reference
    // "null", "List<String>" are plain text - the "<String>" may even
    // get swallowed by an HTML-unaware renderer
    return List.of();
}
```

## Good

```java
/**
 * Converts the given value using the supplied {@link ValueConverter}.
 *
 * <p>Pass {@code null} as {@code converter} to use the default
 * converter returned by {@link ValueConverter#getDefault()}.
 *
 * @param value the value to convert
 * @param converter the converter to apply, or {@code null} for the default
 * @return a {@code List<String>} of converted results, or an empty list
 *     if {@code value} produces no results
 */
public List<String> convert(Object value, ValueConverter converter) {
    return List.of();
}
```

## `{@link}` vs `{@linkplain}` vs `see`

`{@link}` renders the reference in code font, `{@linkplain}` renders it in plain text (useful mid-sentence when code font would be visually jarring), and the `@see` tag creates a separate "See Also" entry rather than an inline reference:

```java
/**
 * Applies validation rules configured by {@linkplain ValidationConfig
 * the active validation configuration}.
 *
 * @see ValidationConfig
 * @see #validateAll(List)
 */
public boolean validate(Object target) {
    return true;
}
```

Prefer `{@link}` to a bare type name every time the referenced type is part of the public API and the reader would plausibly want to navigate to it; reserve plain text for well-known JDK types like `String` or `int` where a link adds little value.

## See Also

- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Document all public API with Javadoc
- [`doc-javadoc-param-return-throws`](doc-javadoc-param-return-throws.md) - Document @param/@return/@throws
- [`doc-javadoc-code-samples`](doc-javadoc-code-samples.md) - Include runnable code samples in Javadoc
