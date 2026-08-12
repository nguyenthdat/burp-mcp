# modern-text-blocks

> Use text blocks for multi-line strings

## Why It Matters

Building multi-line strings with concatenated `"..." + "\n"` literals is hard to read, easy to get wrong (missing a space at a join point, mismatched quote escaping), and turns simple JSON/SQL/HTML snippets into a wall of escape characters. Text blocks (`"""`, finalized in Java 15) preserve the natural layout of the content, handle indentation automatically, and only require escaping the characters that are truly special.

## Bad

```java
String json = "{\n" +
        "  \"name\": \"" + name + "\",\n" +
        "  \"age\": " + age + "\n" +
        "}";

String sql = "SELECT id, name, email " +
        "FROM users " +
        "WHERE active = true " +
        "ORDER BY name";

String html = "<html>\n" +
        "  <body>\n" +
        "    <p>Hello, " + user + "!</p>\n" +
        "  </body>\n" +
        "</html>";
```

## Good

```java
String json = """
        {
          "name": "%s",
          "age": %d
        }
        """.formatted(name, age);

String sql = """
        SELECT id, name, email
        FROM users
        WHERE active = true
        ORDER BY name
        """;

String html = """
        <html>
          <body>
            <p>Hello, %s!</p>
          </body>
        </html>
        """.formatted(user);
```

## Indentation Rules

The compiler strips "incidental" leading whitespace based on the least-indented line (including the closing `"""` delimiter's position), so aligning the closing delimiter controls the block's indentation:

```java
String indented = """
        line one
        line two
        """; // closing """ at this indentation strips that much leading whitespace from every line

String noTrailingNewline = """
        single line, no trailing newline""";
```

Use `\` at the end of a line inside a text block to suppress an unwanted line break, and `\s` to preserve trailing spaces that would otherwise be stripped.

## See Also

- [`modern-var-local-inference`](modern-var-local-inference.md) - Another Java 10+/15+ era readability feature often used together
- [`doc-javadoc-public-api`](doc-javadoc-public-api.md) - Text blocks are useful for embedding example code in Javadoc
- [`api-fluent-method-chaining`](api-fluent-method-chaining.md) - `.formatted()` chains well with text blocks for templating
