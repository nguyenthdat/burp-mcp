# node-path-join

> Use `path.join()` or `path.resolve()` over manual string concatenation for path construction

## Why It Matters

Manual path construction with string concatenation (`dir + '/' + file`) produces incorrect paths when components have trailing slashes, missing slashes, or use different separators. On Windows, paths use `\` not `/`. `path.join()` handles platform-specific separators and normalizes the result correctly. It also prevents double-slashes and dots.

## Bad

```js
// Manual concatenation — fragile, platform-dependent
const filePath = baseDir + '/' + subDir + '/' + filename;
// What if baseDir already has a trailing slash? → double slash
// What if subDir is empty? → trailing slash
// Windows: wrong separator

const configPath = __dirname + '/../config/settings.json';
// Works on Unix, wrong on Windows
```

## Good

```js
import { join, resolve } from 'node:path';

// path.join — handles slashes, empty components, dots
const filePath = join(baseDir, subDir, filename);

// path.resolve — resolves to absolute path
const configPath = resolve(import.meta.dirname, '../config/settings.json');

// Safe file resolution within a base directory
const uploadDir = resolve('./uploads');
const safePath = resolve(uploadDir, userFilename);

// Verify the path stays within uploads/
if (!safePath.startsWith(uploadDir)) {
  throw new Error('Path traversal detected');
}
```

## When Exceptions Apply

URL path construction should use the `URL` constructor, not `path.join()`. Path operations are for filesystem paths.

## See Also

- [sec-path-traversal](./sec-path-traversal.md) - Prevent directory traversal
- [node-url-over-strings](./node-url-over-strings.md) - Use URL constructor for URLs
