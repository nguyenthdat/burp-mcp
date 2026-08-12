# sec-prototype-pollution

> Guard against prototype pollution — the silent JavaScript vulnerability

## Why It Matters

Prototype pollution occurs when an attacker modifies `Object.prototype` or `Array.prototype`, injecting properties that affect all objects in the application. This can lead to RCE, authentication bypass, or denial of service. Vulnerabilities typically arise from recursive merge functions, deep clone utilities, and `Object.assign` with user-controlled input.

## Bad

```js
// Vulnerable deep merge — attacker can set __proto__.isAdmin = true
function merge(target, source) {
  for (const key in source) {
    if (typeof source[key] === 'object') {
      target[key] = merge(target[key] || {}, source[key]);
    } else {
      target[key] = source[key];
    }
  }
  return target;
}

const userInput = JSON.parse(req.body);  // {"__proto__": {"isAdmin": true}}
const config = merge({}, userInput);
// Now every object has isAdmin = true

// Vulnerable Object.assign with user input
Object.assign(existingObject, req.body);
```

## Good

```js
// Safe merge — blocks __proto__, constructor, prototype keys
function safeMerge(target, source) {
  for (const key of Object.keys(source)) {
    if (key === '__proto__' || key === 'constructor' || key === 'prototype') {
      continue;
    }
    const value = source[key];
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      target[key] = safeMerge(target[key] || {}, value);
    } else {
      target[key] = value;
    }
  }
  return target;
}

// Use Object.create(null) for map-like objects
const safeMap = Object.create(null);
safeMap[req.body.key] = req.body.value;  // No prototype to pollute

// Validate JSON input with a schema before merging
import { z } from 'zod';

const ConfigSchema = z.object({
  theme: z.string().optional(),
  language: z.string().optional(),
});

const config = ConfigSchema.parse(req.body);

// Freeze the prototype
Object.freeze(Object.prototype);
Object.freeze(Array.prototype);
```

## Detecting Prototype Pollution

```js
// Monitor for pollution attempts
function detectPollution() {
  const originalToString = {}.toString;
  Object.defineProperty(Object.prototype, 'polluted', {
    set() {
      console.error('Prototype pollution detected!');
      process.exit(1);
    },
  });
}
```

## When Exceptions Apply

In serverless functions or short-lived processes, prototype pollution is less persistent but still dangerous. Always sanitize deeply nested user input.

## See Also

- [fn-immutability](./fn-immutability.md) - Prefer immutable patterns
- [fn-structured-clone](./fn-structured-clone.md) - Safe deep cloning
