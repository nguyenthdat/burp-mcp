# name-verb-function

> Start function names with a verb — functions do things, they're not nouns

## Why It Matters

Functions perform actions. A verb-first name makes the action clear: `getUser()` fetches, `createOrder()` creates, `validateEmail()` validates. Noun-named functions like `user()` or `order()` are ambiguous — do they return the object? Create it? Process it? Verb-first naming eliminates the guesswork.

## Bad

```js
// Noun-named functions — ambiguous purpose
function user(id) { return db.findUser(id); }
function order(data) { return db.createOrder(data); }
function tax(price) { return price * 0.1; }
function email(input) { return /\S+@\S+/.test(input); }
```

## Good

```js
// Verb-first — action is clear
function getUser(id) { return db.findUser(id); }
function createOrder(data) { return db.createOrder(data); }
function calculateTax(price) { return price * 0.1; }
function isValidEmail(input) { return /\S+@\S+/.test(input); }
```

## Common Verb Prefixes

| Verb | Meaning | Example |
|------|---------|---------|
| `get` | Retrieve data | `getUser()`, `getConfig()` |
| `fetch` | Async retrieval | `fetchUsers()`, `fetchData()` |
| `find` | Search/query | `findById()`, `findByEmail()` |
| `create` | Create new | `createUser()`, `createOrder()` |
| `update` | Modify existing | `updateProfile()`, `updateStatus()` |
| `delete` | Remove | `deleteUser()`, `deleteFile()` |
| `save` | Persist | `saveRecord()`, `saveConfig()` |
| `validate` | Check validity | `validateEmail()`, `validateInput()` |
| `parse` | Transform string→data | `parseJSON()`, `parseCSV()` |
| `format` | Transform data→string | `formatDate()`, `formatCurrency()` |
| `compute` | Calculate | `computeHash()`, `computeTotal()` |
| `build` | Construct | `buildQuery()`, `buildResponse()` |

## When Exceptions Apply

Getters (accessors using the `get` keyword) and computed property names are exceptions. Also, well-known lifecycle methods (`constructor`, `init`, `destroy`) don't need verb prefixes.

## See Also

- [name-is-has-bool](./name-is-has-bool.md) - Boolean function prefixes
- [name-handler-verbs](./name-handler-verbs.md) - Handler prefixes
