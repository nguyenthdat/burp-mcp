# mod-named-over-default

> Prefer named exports over default exports

## Why It Matters

Named exports encourage consistent naming, enable better IDE autocompletion and refactoring, and prevent the common problem of consumers importing with arbitrary local names. Default exports make tree-shaking harder for bundlers and break when the module author renames the export. Named exports are self-documenting.

## Bad

```js
// Default export — consumer can name it anything
export default class UserService {
  async findById(id) { /* ... */ }
}

// Consumer has inconsistent naming
import UserService from './user-service.js';
import UserSvc from './user-service.js';
import Service from './user-service.js';  // All valid, all confusing
```

## Good

```js
// Named export — consistent, self-documenting
export class UserService {
  async findById(id) { /* ... */ }
}

// Consumer uses exactly the exported name
import { UserService } from './user-service.js';
```

## Exceptions: Acceptable Default Exports

```js
// 1. Single-export modules where the default is the obvious name
// user-service.js
export default class UserService { /* ... */ }

// 2. Framework components (React, Vue SFC)
export default function MyComponent() { /* ... */ }

// 3. Configuration objects
export default {
  port: 3000,
  dbUrl: process.env.DATABASE_URL,
};

// 4. Main entry point of a package
export { App } from './app.js';
export { default as App } from './app.js';  // Dual export for compatibility
```

## Re-export Pattern

```js
// index.js — barrel file
export { UserService } from './user-service.js';
export { OrderService } from './order-service.js';
// Consumers: import { UserService, OrderService } from './services/index.js';
```

## When Exceptions Apply

Use default exports for framework components and for modules that genuinely export a single primary thing. Prefer named exports for utility modules, services, and libraries.

## See Also

- [mod-barrel-files](./mod-barrel-files.md) - Barrel file best practices
- [mod-no-barrel-re-export-star](./mod-no-barrel-re-export-star.md) - Avoid `export * from`
