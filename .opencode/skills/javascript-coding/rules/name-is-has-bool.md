# name-is-has-bool

> Prefix booleans with `is`, `has`, `should`, or `can` for clarity

## Why It Matters

Boolean variables and functions answer yes/no questions. Without a prefix, the name is ambiguous: does `user.active()` return the active state, or does it activate the user? Prefixing with `is`/`has`/`can` makes the boolean nature immediately obvious and reads naturally in conditions: `if (user.isActive())`.

## Bad

```js
// Ambiguous — is it checking or setting?
if (user.active()) { }
if (config.debug()) { }
if (order.deleted) { }

// Reads unnaturally
if (user.admin) { }      // "if user admin" — not a question
if (file.exists) { }     // "if file exists" — sounds like an action
```

## Good

```js
// Boolean prefixes — reads like a question
if (user.isActive()) { }
if (config.isDebug()) { }
if (order.isDeleted) { }

// Functions answering questions
function isAdmin(user) { return user.role === 'admin'; }
function hasPermission(user, perm) { return user.permissions.includes(perm); }
function canEdit(user, resource) { return user.id === resource.ownerId; }
function shouldRetry(error) { return error.retryable; }
```

## Common Prefixes

| Prefix | Use For | Example |
|--------|---------|---------|
| `is` | State/identity check | `isValid()`, `isEmpty()`, `isAdmin` |
| `has` | Possession/containment | `hasPermission()`, `hasChildren()`, `hasValue` |
| `can` | Capability/permission | `canRead()`, `canExecute()` |
| `should` | Recommendation/policy | `shouldRetry()`, `shouldCache()` |
| `needs` | Requirement | `needsUpdate()`, `needsAuth()` |

## Boolean Variables

```js
// Good naming
const isLoading = true;
const hasError = false;
const isEnabled = true;
const shouldShowModal = false;

// Bad naming
const loading = true;  // Is it a noun (the loading state) or adjective?
const error = false;   // Is it an error object or a flag?
```

## When Exceptions Apply

Well-established names like `debug`, `verbose`, `strict` don't need prefixes when they're clearly booleans by context. But when in doubt, add the prefix.

## See Also

- [name-verb-function](./name-verb-function.md) - Verb-first function names
- [name-camelCase](./name-camelCase.md) - camelCase convention
