# sec-path-traversal

> Validate file paths — reject `../` escapes and ensure paths stay within allowed directories

## Why It Matters

Path traversal (directory traversal) attacks use `../` sequences to access files outside the intended directory. Without validation, an attacker can read sensitive files (`/etc/passwd`, `.env`, SSH keys) or overwrite critical application files. Always resolve and validate paths against a trusted base directory.

## Bad

```js
// Path traversal — attacker can read any file
app.get('/files', (req, res) => {
  const filename = req.query.file;  // "../../../etc/passwd"
  const filePath = `./uploads/${filename}`;
  res.sendFile(filePath);
});

// No base directory enforcement
function readUserFile(filename) {
  return readFileSync(filename, 'utf8');  // Absolute path bypass
}
```

## Good

```js
import { resolve, normalize } from 'node:path';

const UPLOAD_DIR = resolve('./uploads');

app.get('/files', (req, res) => {
  const filename = req.query.file;

  // Reject empty or special characters
  if (!filename || filename.includes('\0')) {
    return res.status(400).json({ error: 'Invalid filename' });
  }

  const safePath = resolve(UPLOAD_DIR, filename);

  // Ensure the resolved path stays within UPLOAD_DIR
  if (!safePath.startsWith(UPLOAD_DIR + '/')) {
    return res.status(403).json({ error: 'Access denied' });
  }

  // Check if file exists within the allowed directory
  if (!existsSync(safePath)) {
    return res.status(404).json({ error: 'File not found' });
  }

  res.sendFile(safePath);
});
```

## Reusable Path Validator

```js
function safeResolve(baseDir, userPath) {
  const resolved = resolve(baseDir, userPath);
  const normalizedBase = resolve(baseDir) + '/';

  if (!resolved.startsWith(normalizedBase)) {
    throw new Error('Path traversal detected');
  }

  return resolved;
}

// Usage
const filePath = safeResolve('./uploads', req.query.file);
res.sendFile(filePath);
```

## When Exceptions Apply

Path validation is mandatory whenever file paths are derived from user input — URL parameters, form fields, uploaded filenames. Internal paths constructed from trusted data (e.g., config values) can skip validation.

## See Also

- [sec-input-sanitize](./sec-input-sanitize.md) - Sanitize all user input
- [node-path-join](./node-path-join.md) - Use path.join() for path construction
