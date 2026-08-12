# anti-non-null-assertion-abuse

> Don't overuse the `!` non-null assertion operator

## Why It Matters

Every `!` is an unchecked promise to the compiler that a value isn't `null`/`undefined` — and unlike a real narrowing check, that promise carries no runtime evidence. Overusing it (especially in codebases migrating from a looser type-checking regime) reintroduces the exact class of null-reference bugs that `strictNullChecks` was adopted to eliminate, just moved from "compiler catches it" back to "user hits a crash in production." A codebase littered with `!` also signals, to any reader, that the types can't be trusted at face value — which erodes the main benefit of using TypeScript at all.

## Bad

```typescript
function getConfigValue(key: string) {
  const value = configMap.get(key)!; // asserts the key always exists
  return value.trim();
}

function renderUser(id: string) {
  const el = document.getElementById(id)!; // DOM element might not exist
  el.textContent = 'Loaded';
}

// Assertions chained through multiple steps compound the risk
const width = ref.current!.getBoundingClientRect()!.width;
```

## Good

```typescript
function getConfigValue(key: string): string {
  const value = configMap.get(key);
  if (value === undefined) {
    throw new Error(`Missing required config key: ${key}`);
  }
  return value.trim();
}

function renderUser(id: string) {
  const el = document.getElementById(id);
  if (!el) {
    console.warn(`Element #${id} not found`);
    return;
  }
  el.textContent = 'Loaded';
}

const rect = ref.current?.getBoundingClientRect();
const width = rect?.width ?? 0;
```

## Legitimate, Narrow Uses

`!` is defensible immediately after a check the compiler genuinely cannot correlate (e.g., a `.filter(Boolean)` result the compiler can't narrow through), and it's better replaced by expressing the invariant in the type when possible:

```typescript
// The compiler can't see that filter(Boolean) removes falsy values —
// prefer a typed predicate over a bare assertion
const names = users.map((u) => u.name).filter((n): n is string => n != null);
```

## See Also

- [lint-no-non-null-assertion](lint-no-non-null-assertion.md) - Enable `@typescript-eslint/no-non-null-assertion`
- [type-narrow-guards](type-narrow-guards.md) - Use type guards to narrow union types safely
- [type-strict-null-checks](type-strict-null-checks.md) - Rely on `strictNullChecks` instead of manual null checks everywhere
