# proj-no-giant-files

> Keep files under 300 lines — split large files into focused modules

## Why It Matters

Files over 300 lines are harder to navigate, review, and test. They accumulate responsibilities, making refactoring risky. A file with a single responsibility rarely exceeds 300 lines. Splitting large files forces you to define clear module boundaries, making the codebase easier to understand and maintain.

## Bad

```js
// user-service.js — 800 lines
export function createUser() { /* 50 lines */ }
export function updateUser() { /* 50 lines */ }
export function deleteUser() { /* 50 lines */ }
export function findUser() { /* 50 lines */ }
export function validateEmail() { /* 30 lines */ }
export function hashPassword() { /* 30 lines */ }
export function sendWelcomeEmail() { /* 100 lines */ }
export function generateToken() { /* 60 lines */ }
// ... 400 more lines
```

## Good

```js
// user-service.js — 80 lines, delegates to focused modules
import { create, update, remove, findById } from './user-repository.js';
import { validateEmail } from './validators.js';
import { hashPassword } from './crypto.js';
import { sendWelcomeEmail } from './email-service.js';
import { generateToken } from './auth.js';

export async function createUser(input) {
  validateEmail(input.email);
  const hashed = await hashPassword(input.password);
  const user = await create({ ...input, password: hashed });
  await sendWelcomeEmail(user.email);
  return user;
}
```

## Enforce with ESLint

```json
// eslint.config.mjs
{
  "rules": {
    "max-lines": ["warn", { "max": 300, "skipBlankLines": true, "skipComments": true }]
  }
}
```

## When Exceptions Apply

Generated code, configuration files, and comprehensive test files for a single module may exceed 300 lines. The rule is a guideline, not a hard limit — apply judgment.

## See Also

- [proj-layer-architecture](./proj-layer-architecture.md) - Layer architecture
- [mod-separate-concerns](./mod-separate-concerns.md) - One module, one responsibility
