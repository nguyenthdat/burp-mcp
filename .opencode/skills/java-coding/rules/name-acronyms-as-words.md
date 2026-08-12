# name-acronyms-as-words

> Treat acronyms as words: `HttpClient`, not `HTTPClient`

## Why It Matters

The Google Java Style Guide explicitly treats acronyms as ordinary words for casing purposes, because fully capitalized acronyms clash with `camelCase`/`PascalCase` boundary rules and produce ambiguous runs of capitals when combined (`XMLHTTPRequestParser` is nearly unreadable). Treating an acronym as a word keeps case transitions predictable, which matters for both human scanning and case-based tooling like `IntelliJ`'s "camelCase" search and Checkstyle's naming rules.

## Bad

```java
public class HTTPClient {  // ambiguous capital run

    public URLConnection openURLConnection(String urlString) {
        // ...
        return null;
    }

    public String parseJSONResponse(String body) {  // JSONResponse reads as one blob
        return body;
    }

    private String userID;   // looks like a constant, not a field
    private String XMLPayload;
}
```

## Good

```java
public class HttpClient {

    public UrlConnection openUrlConnection(String urlString) {
        // ...
        return null;
    }

    public String parseJsonResponse(String body) {
        return body;
    }

    private String userId;
    private String xmlPayload;
}
```

## Two-Letter Acronyms and Leading Position

Two-letter acronyms (`ID`, `IO`, `UI`) follow the same rule; only the first letter is capitalized mid-name, and the whole acronym is lowercase when it starts a `camelCase` identifier.

```java
public class IoUtils {          // not IOUtils
    public static String readId(InputStream is) {  // param stays lowercase-first
        return null;
    }
}

int id;          // fine at variable position, all-lowercase
UiComponent ui;  // "Ui" not "UI" mid/start of a type name
```

The one broadly accepted exception is when an acronym is itself the entire identifier and is very short and universally recognized (e.g., a local variable literally named `id` or `db`) - there the word-casing rule and plain-English usage coincide, so there is nothing to fix.

## See Also

- [`name-classes-pascal`](name-classes-pascal.md) - Use PascalCase for classes, interfaces, enums, records
- [`name-methods-camel`](name-methods-camel.md) - Use camelCase for methods and fields
- [`name-no-hungarian-notation`](name-no-hungarian-notation.md) - Avoid Hungarian notation and type-suffix cruft
