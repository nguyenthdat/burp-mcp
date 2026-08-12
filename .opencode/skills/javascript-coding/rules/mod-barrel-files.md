# mod-barrel-files

> Use barrel files (index.js) sparingly — they can defeat tree-shaking

## Why It Matters

Barrel files (`index.js` that re-exports from other modules) provide a clean import path, but they can cause problems: bundlers may include unused modules (defeating tree-shaking), circular dependency risks increase, and module load order becomes harder to reason about. Use barrels intentionally and avoid deep nesting.

## Bad

```js
// Barrel that re-exports everything unconditionally
// services/index.js
export { UserService } from './user-service.js';
export { OrderService } from './order-service.js';
export { PaymentService } from './payment-service.js';
export { NotificationService } from './notification-service.js';
// Importing just UserService forces all four modules to be evaluated
```

## Good

```js
// Import directly from the source module
import { UserService } from './services/user-service.js';

// Barrel is fine for a small, co-located set of related exports
// utils/index.js
export { formatDate } from './format-date.js';
export { parseDate } from './parse-date.js';
export { isValidDate } from './is-valid-date.js';

// Consumers can import from one place
import { formatDate, parseDate } from './utils/index.js';
```

## Barrel File Best Practices

```js
// Prefer direct imports for large, loosely related modules
import { UserService } from './services/user-service.js';
import { OrderService } from './services/order-service.js';

// Use barrels for tightly coupled, small utility collections
// constants/index.js
export { API_URL, MAX_RETRIES } from './api.js';
export { COLORS, FONT_SIZES } from './theme.js';
```

## When Exceptions Apply

Barrel files are useful for:
- Public API surface of a library (defined in `exports` field of package.json)
- Small utility collections (< 5 modules)
- Framework components where import ergonomics matter more than bundle size

## See Also

- [mod-no-barrel-re-export-star](./mod-no-barrel-re-export-star.md) - Avoid `export * from`
- [mod-export-near-definition](./mod-export-near-definition.md) - Export close to definitions
