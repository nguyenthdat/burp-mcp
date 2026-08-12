# doc-see-references

> Use `@see` to cross-reference related functions, modules, or external documentation

## Why It Matters

No function exists in isolation. `@see` tags connect related concepts, helping developers discover the full API surface. When a function delegates to another or has a related counterpart, cross-references prevent developers from missing important context. They also link to external specs, RFCs, and ADRs.

## Bad

```js
/**
 * Creates a new user.
 */
export async function createUser(data) {
  // No mention that deleteUser exists, or that there's a bulk version
}

/**
 * Removes a user.
 */
export async function deleteUser(id) {
  // No mention that createUser exists
}
```

## Good

```js
/**
 * Creates a new user account.
 *
 * @param {UserInput} data - User creation data
 * @returns {Promise<User>} The created user
 * @throws {ValidationError} If the input is invalid
 * @see {@link updateUser} for modifying existing users
 * @see {@link deleteUser} for removing users
 * @see {@link createUsers} for bulk creation
 * @see {@link https://tools.ietf.org/html/rfc5322} Email format spec
 */
export async function createUser(data) { /* ... */ }

/**
 * Updates an existing user's profile.
 *
 * @see {@link createUser} for creating new users
 * @see {@link UserSchema} for valid field definitions
 */
export async function updateUser(id, changes) { /* ... */ }
```

## Cross-Reference Patterns

```js
/**
 * @see {@link someFunction} — Link to another function
 * @see {@link SomeClass} — Link to a class
 * @see {@link SomeClass#method} — Link to a method
 * @see {@link module:utils} — Link to a module
 * @see {@link https://example.com/spec} — Link to external URL
 * @see {@link ./ARCHITECTURE.md} — Link to a local file
 */

// In a bug fix comment
/**
 * Workaround for the event loop blocking issue.
 * @see {@link https://github.com/nodejs/node/issues/12345}
 */
```

## When Exceptions Apply

Don't add `@see` for every tangentially related function. Cross-reference when knowing about the other function meaningfully helps the developer using this one.

## See Also

- [doc-jsdoc-public](./doc-jsdoc-public.md) - JSDoc for public APIs
- [doc-deprecated-tag](./doc-deprecated-tag.md) - Deprecation with migration guidance
