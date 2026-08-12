# sec-input-sanitize

> Always sanitize and validate user input before processing

## Why It Matters

User input is the primary attack vector for XSS, SQL injection, command injection, and path traversal. Never trust input from HTTP requests, file uploads, CLI arguments, or any external source. Sanitize at the boundary and work with validated data internally.

## Bad

```js
// Unsanitized input used directly
function searchUsers(req, res) {
  const query = req.query.q;
  const results = db.query(`SELECT * FROM users WHERE name LIKE '%${query}%'`);
  res.render('results', { query, results });  // XSS if query contains HTML
}

// No validation on file paths
function readFile(req, res) {
  const filename = req.query.file;
  const content = readFileSync(`./uploads/${filename}`, 'utf8');
  res.send(content);
}
```

## Good

```js
function searchUsers(req, res) {
  const query = String(req.query.q).trim().slice(0, 100);

  // Use parameterized query to prevent SQL injection
  const results = db.query(
    'SELECT * FROM users WHERE name LIKE ?',
    [`%${query.replace(/[%_]/g, '\\$&')}%`],
  );

  // Escape HTML output to prevent XSS
  res.render('results', {
    query: escapeHtml(query),
    results,
  });
}

function readFile(req, res) {
  const filename = req.query.file;

  // Validate: no path traversal, no special chars
  if (!/^[a-zA-Z0-9_.-]+$/.test(filename)) {
    return res.status(400).json({ error: 'Invalid filename' });
  }

  const safePath = path.resolve('./uploads', filename);
  if (!safePath.startsWith(path.resolve('./uploads'))) {
    return res.status(403).json({ error: 'Path traversal denied' });
  }

  const content = await readFile(safePath, 'utf8');
  res.send(content);
}
```

## Input Sanitization Checklist

- **Strings**: Trim, limit length, escape for context (HTML/JS/SQL/shell)
- **Numbers**: Parse with `Number()` or `parseInt()`, check for NaN, clamp range
- **Files**: Validate extension, MIME type, size; use `path.resolve()` to prevent traversal
- **Email/URL**: Use `new URL()` to parse; validate format with regex or libraries
- **JSON**: Parse with `JSON.parse()` wrapped in try/catch; validate schema

## See Also

- [sec-path-traversal](./sec-path-traversal.md) - Prevent directory traversal
- [sec-sql-injection](./sec-sql-injection.md) - Parameterized queries
- [sec-input-size-limits](./sec-input-size-limits.md) - Limit input sizes
