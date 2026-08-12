# name-UPPER_SNAKE

> Use UPPER_SNAKE_CASE for true constants — values that are known at load time and never change

## Why It Matters

UPPER_SNAKE_CASE signals that a value is a constant: it won't be reassigned and its value is fixed for the lifetime of the process. This helps readers distinguish between runtime variables and configuration/constant values at a glance. It's also the convention in most programming languages, making code more familiar to polyglot developers.

## Bad

```js
// Constants in camelCase — indistinguishable from variables
const maxRetries = 3;
const apiBaseUrl = 'https://api.example.com';
const defaultTimeoutMs = 5000;

// Mutable variables in UPPER_SNAKE — misleading
let MAX_USERS = 100;
MAX_USERS = 200;
```

## Good

```js
// True constants — UPPER_SNAKE_CASE
const MAX_RETRIES = 3;
const API_BASE_URL = 'https://api.example.com';
const DEFAULT_TIMEOUT_MS = 5000;

// Enum-like constant groups
const ORDER_STATUS = Object.freeze({
  PENDING: 'pending',
  PROCESSING: 'processing',
  SHIPPED: 'shipped',
  DELIVERED: 'delivered',
});

// Configuration
const CONFIG = Object.freeze({
  PORT: 3000,
  HOST: 'localhost',
  MAX_BODY_SIZE: '1mb',
});

// Regex patterns
const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
```

## What Does NOT Get UPPER_SNAKE_CASE

```js
// Runtime-computed values — even if they don't change after assignment
const currentTimestamp = Date.now();  // camelCase — not a "true" constant
const env = getEnvironment();         // camelCase — computed
const userCount = users.length;       // camelCase — derived at runtime

// Module-level mutable references
const logger = createLogger();  // camelCase — the const reference is fixed but...
// logger is used as a variable throughout the code
```

## When Exceptions Apply

The distinction between "true constant" and "const variable" is a convention. Some teams use UPPER_SNAKE_CASE for all module-level `const` declarations. Be consistent with your team.

## See Also

- [name-camelCase](./name-camelCase.md) - camelCase for variables
- [type-no-magic-strings](./type-no-magic-strings.md) - Constants over magic strings
