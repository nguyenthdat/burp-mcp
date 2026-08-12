# name-PascalCase

> Use PascalCase for classes and constructor functions

## Why It Matters

PascalCase distinguishes constructors (which must be called with `new`) from regular functions. This convention is so deeply ingrained that linters warn when a PascalCase function is called without `new`. It also aligns with the naming of built-in classes (`Promise`, `Array`, `Map`, `Error`).

## Bad

```js
// Inconsistent class naming
class userService {
  findById(id) { /* ... */ }
}

class Order_Processor {
  process(order) { /* ... */ }
}

function apiClient(baseUrl) {  // Looks like a constructor but isn't PascalCase
  return { baseUrl };
}
```

## Good

```js
// PascalCase for classes and constructors
class UserService {
  findById(id) { /* ... */ }
}

class OrderProcessor {
  process(order) { /* ... */ }
}

class HttpClient { }

// Constructor functions (prefer class syntax)
function ApiClient(baseUrl) {
  if (!new.target) throw new Error('Use new ApiClient()');
  this.baseUrl = baseUrl;
}
```

## What Gets PascalCase

```js
// Classes
class UserController { }
class DatabaseConnection { }

// Custom error classes
class ValidationError extends Error { }
class NotFoundError extends Error { }

// React/Vue components
function UserProfile(props) { /* ... */ }  // Capitalized function components

// Enums (const-like objects as namespaces)
const OrderStatus = {
  PENDING: 'pending',
  SHIPPED: 'shipped',
};
// OrderStatus is PascalCase, values are lowercase or UPPER_SNAKE
```

## What Does NOT Get PascalCase

```js
// Regular functions — camelCase
function fetchUser(id) { /* ... */ }

// Factory functions — camelCase
function createUser(data) { /* ... */ }

// Modules/files — kebab-case
// file: user-service.js

// Variables — camelCase
const userService = new UserService();
```

## See Also

- [name-camelCase](./name-camelCase.md) - camelCase for variables/functions
- [name-kebab-case](./name-kebab-case.md) - kebab-case for files
