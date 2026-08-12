# fn-template-literals

> Use template literals over string concatenation

## Why It Matters

Template literals provide embedded expressions, multi-line strings, and tagged templates in a single syntax. String concatenation with `+` is error-prone (missing spaces, type coercion issues) and hard to read with multiple variables. Template literals make string construction declarative and self-documenting.

## Bad

```js
const message = 'Hello, ' + user.name + '! You have ' + count + ' new messages.';

const url = baseUrl + '/api/' + version + '/users/' + userId;

const query = 'SELECT * FROM ' + table + ' WHERE ' + column + ' = ' + value;

const html = '<div class="' + className + '">\n' +
             '  <h1>' + title + '</h1>\n' +
             '</div>';
```

## Good

```js
const message = `Hello, ${user.name}! You have ${count} new messages.`;

const url = `${baseUrl}/api/${version}/users/${userId}`;

// Never for SQL queries — use parameterized queries instead
// See sec-sql-injection rule

const html = `
<div class="${className}">
  <h1>${title}</h1>
</div>
`;
```

## Tagged Templates

```js
// Custom string processing
function highlight(strings, ...values) {
  return strings.reduce((result, str, i) =>
    `${result}${str}<strong>${values[i] ?? ''}</strong>`, '');
}

const result = highlight`Found ${count} results for "${query}"`;
// Found <strong>5</strong> results for "<strong>javascript</strong>"
```

## When Exceptions Apply

Don't use template literals for SQL queries (parameterized queries only). For very simple single-variable concatenation (`'Hello ' + name`), either style is fine — template literals are still preferred for consistency.

## See Also

- [sec-sql-injection](./sec-sql-injection.md) - Never use template literals for SQL
- [perf-avoid-string-concat-loops](./perf-avoid-string-concat-loops.md) - Performance in loops
