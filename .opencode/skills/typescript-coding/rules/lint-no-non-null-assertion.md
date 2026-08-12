# lint-no-non-null-assertion

> Enable `@typescript-eslint/no-non-null-assertion`

## Why It Matters

The `!` non-null assertion operator tells the compiler "trust me, this is never `null`/`undefined`" without any runtime check — if the assumption is wrong, the result is a `TypeError` at the point of use, often far from where the bad value actually originated, with a stack trace that gives no indication *why* the value was supposed to be non-null. It's a compile-time-only claim that bypasses the exact safety `strictNullChecks` exists to provide. This rule doesn't ban all uses (some are genuinely safe) but forces every occurrence to be visible in review and, ideally, replaced with a real check or a narrowing guard.

## Bad

```typescript
function getUser(id: string): User | undefined {
  return users.find((u) => u.id === id);
}

function greet(id: string) {
  const user = getUser(id)!; // asserts non-null with no evidence
  console.log(`Hello, ${user.name}`); // throws if id doesn't match any user
}
```

## Good

```javascript
// eslint.config.js
export default tseslint.config({
  rules: {
    '@typescript-eslint/no-non-null-assertion': 'error',
  },
});
```

```typescript
function greet(id: string) {
  const user = getUser(id);
  if (!user) {
    throw new Error(`No user found for id: ${id}`);
  }
  console.log(`Hello, ${user.name}`); // narrowed, no assertion needed
}

// Or handle the missing case explicitly
function greet(id: string) {
  const user = getUser(id) ?? { name: 'Guest' };
  console.log(`Hello, ${user.name}`);
}
```

## When `!` Is Genuinely Safe

Some cases are provably safe but the compiler can't see it — e.g. right after a `.length` check on an array literal it can't narrow across:

```typescript
const first = items[0]!; // safe only if you've already checked items.length > 0 above
```

Prefer expressing the invariant so the compiler enforces it instead (a non-empty-array type, `at(0)` with an explicit check), and reserve `!` with an inline disable comment explaining the invariant for the rare case where that isn't practical.

## See Also

- [anti-non-null-assertion-abuse](anti-non-null-assertion-abuse.md) - Don't overuse the `!` non-null assertion operator
- [type-narrow-guards](type-narrow-guards.md) - Use type guards to narrow union types safely
- [lint-no-unchecked-indexed-access](lint-no-unchecked-indexed-access.md) - Enable `noUncheckedIndexedAccess` in `tsconfig.json`
