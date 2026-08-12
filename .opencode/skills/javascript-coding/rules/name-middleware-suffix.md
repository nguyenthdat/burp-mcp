# name-middleware-suffix

> Suffix Express/Koa middleware functions with `Middleware` for clarity

## Why It Matters

Middleware functions have a specific signature and role in the request pipeline. Naming them with the `Middleware` suffix (or storing them in a `middleware/` directory) makes their role explicit. This prevents confusion with regular handlers or utility functions and makes the middleware stack auditable.

## Bad

```js
// Ambiguous — is this a handler, middleware, utility?
function auth(req, res, next) { /* ... */ }
function logger(req, res, next) { /* ... */ }
function validate(req, res, next) { /* ... */ }
function errorHandler(err, req, res, next) { /* ... */ }

app.use(auth);
app.use(logger);
app.post('/users', validate, createUser);
app.use(errorHandler);
```

## Good

```js
// Explicit middleware naming
function authenticateMiddleware(req, res, next) { /* ... */ }
function requestLoggerMiddleware(req, res, next) { /* ... */ }
function validateBodyMiddleware(req, res, next) { /* ... */ }
function errorHandlerMiddleware(err, req, res, next) { /* ... */ }

app.use(authenticateMiddleware);
app.use(requestLoggerMiddleware);
app.post('/users', validateBodyMiddleware, createUser);
app.use(errorHandlerMiddleware);

// Or: store in middleware/ directory and export clearly
// middleware/auth.js
export function authenticate(req, res, next) { /* ... */ }

// Usage:
import { authenticate } from './middleware/auth.js';
app.use(authenticate);  // Import path makes it clear it's middleware
```

## When Exceptions Apply

When middleware is organized in a `/middleware` directory, the suffix is redundant — the import path signals the role. Use the suffix when middleware is colocated with other code.

## See Also

- [proj-middleware-stack](./proj-middleware-stack.md) - Compose middleware in dedicated files
- [proj-layer-architecture](./proj-layer-architecture.md) - Layer architecture
