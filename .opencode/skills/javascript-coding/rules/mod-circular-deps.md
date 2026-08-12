# mod-circular-deps

> Avoid circular dependencies — they cause undefined exports and runtime errors

## Why It Matters

Circular dependencies (module A imports from B, B imports from A) create a chicken-and-egg problem. At runtime, one module will see an incomplete or undefined version of the other. This leads to `TypeError: x is not a function` or `Cannot read properties of undefined` errors that are notoriously hard to debug.

## Bad

```js
// user-service.js
import { sendEmail } from './email-service.js';

export async function createUser(data) {
  await saveUser(data);
  await sendEmail(data.email, 'Welcome!');  // Might be undefined
}

// email-service.js
import { createUser } from './user-service.js';  // Circular!

export async function sendEmail(to, body) {
  // ...
}
```

## Good

```js
// Extract shared types/interfaces into a separate module
// user-types.js
export function validateEmail(email) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
}

// user-service.js
import { validateEmail } from './user-types.js';
import { sendEmail } from './email-service.js';  // No circular dep

export async function createUser(data) {
  if (!validateEmail(data.email)) throw new Error('Invalid email');
  await saveUser(data);
  await sendEmail(data.email, 'Welcome!');
}

// email-service.js
import { validateEmail } from './user-types.js';  // Shared module — no circular

export async function sendEmail(to, body) {
  if (!validateEmail(to)) throw new Error('Invalid recipient');
  // ...
}
```

## Breaking Circular Dependencies

```js
// 1. Extract shared logic into a third module
// 2. Use dependency injection
export function createUserService({ emailService }) {
  return {
    async createUser(data) {
      await emailService.sendEmail(data.email, 'Welcome!');
    },
  };
}

// 3. Use dynamic imports (last resort)
export async function createUser(data) {
  const { sendEmail } = await import('./email-service.js');
  await sendEmail(data.email, 'Welcome!');
}
```

## When Exceptions Apply

Circular dependencies within the same directory for closely related types are sometimes unavoidable. In those cases, use the third-module extraction pattern.

## See Also

- [mod-separate-concerns](./mod-separate-concerns.md) - One module, one responsibility
- [mod-dynamic-import](./mod-dynamic-import.md) - Dynamic import() for lazy loading
