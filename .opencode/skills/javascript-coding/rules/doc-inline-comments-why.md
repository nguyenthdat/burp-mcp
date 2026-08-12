# doc-inline-comments-why

> Comment WHY the code exists, not WHAT it does — the code already says WHAT

## Why It Matters

Comments that restate the code (`// increment counter` above `i++`) add noise, not value. Comments that explain WHY a non-obvious choice was made preserve institutional knowledge: business rules, performance workarounds, known limitations. These are things the code cannot express and are lost without documentation.

## Bad

```js
// Increment the counter by 1
counter++;

// Loop through the array
for (const item of items) {
  // Process each item
  process(item);
}

// Set the timeout to 5 seconds
setTimeout(cleanup, 5000);
```

## Good

```js
// Use a Map instead of an object because keys can be arbitrary strings
// from user input, which would collide with Object.prototype properties
const cache = new Map();

// Workaround for Node.js issue #45678: fs.promises.readFile leaks
// file descriptors when called with large files on macOS.
// Remove after Node.js 22.x is released.
const content = await readFileWithRetry(path);

// Rate limit is 100 req/min per API key (provider docs: https://...)
const delay = 600;  // 100ms buffer below the limit

// Intentional use of == null to catch both null and undefined
if (value == null) return defaultValue;
```

## When to Remove Comments

```js
// Remove comments that can be replaced by clearer code
// Before
// Check if the user is an admin
if (user.role === 'admin') { /* ... */ }

// After — rename to a function
if (isAdmin(user)) { /* ... */ }

// Remove commented-out code
// const oldImplementation = () => { ... };  // Git history has this
```

## When Exceptions Apply

For learning resources, tutorials, and educational code, "what" comments are valuable. For production code, "why" comments are the standard.

## See Also

- [doc-no-stale-comments](./doc-no-stale-comments.md) - Remove stale comments
- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
