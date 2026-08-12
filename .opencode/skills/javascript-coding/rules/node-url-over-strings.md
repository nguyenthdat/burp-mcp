# node-url-over-strings

> Use the `URL` constructor over string manipulation for URL parsing and construction

## Why It Matters

Manual string manipulation of URLs is error-prone: query strings, fragments, encoding, and relative paths are all complex to handle correctly. The `URL` constructor (WHATWG URL API) parses URLs according to the spec, provides structured access to components, and handles encoding automatically. It replaces the legacy `url.parse()`.

## Bad

```js
// Manual string manipulation — fragile
function buildUrl(base, path, params) {
  let url = base + '/' + path;
  if (params) {
    const qs = Object.entries(params)
      .map(([k, v]) => k + '=' + v)
      .join('&');
    url += '?' + qs;
  }
  return url;
}

// Legacy url.parse
import { parse } from 'node:url';  // Legacy
const parsed = parse('https://example.com/path?q=1');
```

## Good

```js
// URL constructor — structured, handles encoding
function buildUrl(base, path, params) {
  const url = new URL(path, base);
  if (params) {
    for (const [key, value] of Object.entries(params)) {
      url.searchParams.set(key, value);
    }
  }
  return url.toString();
}

// Parsing URLs
const url = new URL('https://user:pass@example.com:8080/path?q=search#hash');

url.protocol;   // 'https:'
url.hostname;   // 'example.com'
url.port;       // '8080'
url.pathname;   // '/path'
url.search;     // '?q=search'
url.hash;       // '#hash'
url.username;   // 'user'
url.password;   // 'pass'

// Manipulating query parameters
url.searchParams.set('page', '2');
url.searchParams.delete('q');
url.searchParams.get('page');  // '2'
url.toString();  // Full updated URL
```

## When Exceptions Apply

For simple string concatenation of a known base URL with a known path, template literals are fine (`${base}/api/users`). For anything involving query parameters or user input, use `URL`.

## See Also

- [node-path-join](./node-path-join.md) - Path construction for filesystem
- [sec-input-sanitize](./sec-input-sanitize.md) - Sanitize user input
