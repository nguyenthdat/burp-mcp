# api-minimal-surface

> Keep the public API surface as small as the consumer actually needs

## Why It Matters

Every exported name — function, type, class member — becomes a promise to consumers that you'll maintain it, and every one you don't need to export is one fewer thing that can break their code when you refactor. A large, unfocused public surface makes a module's actual contract harder to discover (consumers can't tell what's load-bearing vs. incidental), and it locks in implementation details that should be free to change. Minimizing the surface is the single highest-leverage thing you can do for long-term maintainability of a library or internal module.

## Bad

```typescript
// user-service.ts — everything is exported "just in case"
export interface UserRow {
  id: string;
  email: string;
  passwordHash: string;
  createdAt: Date;
}

export function hashPassword(pw: string): string { /* ... */ }
export function normalizeEmail(email: string): string { /* ... */ }
export function buildUserQuery(filters: unknown): string { /* ... */ }

export async function createUser(email: string, password: string): Promise<UserRow> {
  const normalized = normalizeEmail(email);
  const hash = hashPassword(password);
  return db.insert(buildUserQuery({ email: normalized, hash }));
}
```

## Good

```typescript
// user-service.ts — only the contract consumers need is public
export interface User {
  id: string;
  email: string;
  createdAt: Date;
}

export async function createUser(email: string, password: string): Promise<User> {
  const normalized = normalizeEmail(email);
  const hash = hashPassword(password);
  const row = await db.insert(buildUserQuery({ email: normalized, hash }));
  return toPublicUser(row);
}

// hashPassword, normalizeEmail, buildUserQuery, UserRow, toPublicUser
// stay module-private (no `export`) — free to change without notice.
```

## A Checklist Before Adding `export`

- Does a consumer *outside this module* actually call/reference this today, or is this speculative?
- Could this be inlined into the one place that uses it instead?
- If it's a type, does it leak an internal representation (a raw DB row, a third-party SDK type) that should be mapped to a domain type first?
- Would removing this later count as a breaking change? If yes, exporting it is a deliberate commitment, not a convenience.

## Enforcing This

TypeScript's `isolatedModules`/`noUnusedLocals` won't catch over-exporting, but a bundler's tree-shaking report or `ts-prune` (a tool that finds exported symbols with no external importers) can surface unused public exports for pruning.

## See Also

- [api-named-over-default-export](api-named-over-default-export.md) - Prefer named exports over default exports
- [api-module-boundary-types](api-module-boundary-types.md) - Define explicit DTOs at module/service boundaries, separate from internal domain models
- [proj-module-boundaries](proj-module-boundaries.md) - Keep import graphs acyclic and boundaries explicit
