# anti-magic-numbers

> Don't scatter unexplained magic numbers/strings through code

## Why It Matters

A bare literal like `if (retries > 3)` or `setTimeout(fn, 86400000)` tells the reader nothing about *why* that particular value was chosen, and when the same value needs to change, it's often duplicated across multiple call sites with no way to find them all reliably (grepping for `3` or `86400000` returns unrelated matches). Naming the value as a constant turns an opaque number into documentation: `MAX_RETRY_ATTEMPTS` explains intent at the point of use, and changing it in one place changes it everywhere it's referenced, eliminating the class of bug where two call sites that were supposed to stay in sync silently drift apart.

## Bad

```typescript
async function fetchWithRetry(url: string) {
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      return await fetch(url);
    } catch {
      await new Promise((r) => setTimeout(r, 1000 * (attempt + 1)));
    }
  }
  throw new Error('failed after retries');
}

function isSessionExpired(lastActive: number) {
  return Date.now() - lastActive > 86400000; // what unit? what does this represent?
}
```

## Good

```typescript
const MAX_RETRY_ATTEMPTS = 3;
const BASE_RETRY_DELAY_MS = 1000;

async function fetchWithRetry(url: string) {
  for (let attempt = 0; attempt < MAX_RETRY_ATTEMPTS; attempt++) {
    try {
      return await fetch(url);
    } catch {
      await new Promise((r) => setTimeout(r, BASE_RETRY_DELAY_MS * (attempt + 1)));
    }
  }
  throw new Error(`failed after ${MAX_RETRY_ATTEMPTS} retries`);
}

const SESSION_TIMEOUT_MS = 24 * 60 * 60 * 1000; // 24 hours

function isSessionExpired(lastActive: number) {
  return Date.now() - lastActive > SESSION_TIMEOUT_MS;
}
```

## When a Literal Isn't "Magic"

Not every number needs a name — `array[0]`, `x * 2` for doubling, or `for (let i = 0; ...)` are self-explanatory in context and naming them (`const FIRST_INDEX = 0`) adds noise rather than clarity. The rule targets values whose *meaning* isn't obvious from the surrounding code and that represent a business rule, a limit, a timeout, or a threshold — exactly the kind of value that's likely to be referenced from more than one place or to change later.

```javascript
// eslint rule for enforcement, tuned to avoid flagging trivial literals
{
  "rules": {
    "@typescript-eslint/no-magic-numbers": [
      "warn",
      { "ignore": [-1, 0, 1, 2], "ignoreArrayIndexes": true, "ignoreEnums": true }
    ]
  }
}
```

## See Also

- [name-SCREAMING-const](name-SCREAMING-const.md) - Name constants in SCREAMING_SNAKE_CASE
- [imm-prefer-const](imm-prefer-const.md) - Prefer `const` bindings for values that never change
- [doc-inline-why-not-what](doc-inline-why-not-what.md) - Write comments that explain why, not what
