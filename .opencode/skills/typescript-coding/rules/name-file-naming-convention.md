# name-file-naming-convention

> Apply one consistent file naming convention (kebab-case or PascalCase) per project

## Why It Matters

Inconsistent file naming (`userProfile.ts` next to `user-settings.ts` next to `UserAvatar.ts`) makes files harder to find by pattern, breaks import-path muscle memory, and causes real cross-platform bugs: macOS and Windows filesystems are case-insensitive by default, so `import "./UserCard"` can resolve locally against a file actually named `usercard.ts` and only fail once deployed to a case-sensitive Linux CI/production filesystem. Picking one convention and applying it uniformly avoids both the readability cost and this class of environment-specific import bug.

## Bad

```
src/
  userProfile.ts
  user-settings.ts
  UserAvatar.tsx
  order_history.ts
  ApiClient.ts
  utils.ts
```

Four different casing styles for files that all serve the same kind of role, with no way to predict which convention any given new file will follow.

## Good

```
src/
  user-profile.ts
  user-settings.ts
  user-avatar.tsx
  order-history.ts
  api-client.ts
  utils.ts
```

Or, consistently the other direction if the project prefers matching the exported symbol's casing:

```
src/
  UserProfile.ts
  UserSettings.ts
  UserAvatar.tsx
  OrderHistory.ts
  ApiClient.ts
```

## Two Common, Both-Valid Conventions

| Convention | Typical rationale | Common in |
|---|---|---|
| `kebab-case.ts` | Filesystem-safe, no case-sensitivity risk, matches URL/CLI conventions | Node backends, Angular, most non-React tooling |
| `PascalCase.tsx` | File name mirrors the primary exported class/component name exactly | React/component-heavy codebases (matches the component's identifier 1:1) |

Both are fine choices — the actual requirement is picking **one** and enforcing it project-wide, not which one you pick. Mixing conventions within one repo is the actual anti-pattern.

## Enforcing With a Lint Plugin

```jsonc
// eslint-plugin-unicorn
{
  "rules": {
    "unicorn/filename-case": ["error", { "case": "kebabCase" }]
  }
}
```

For React projects preferring PascalCase component files, configure `unicorn/filename-case` with `cases: { pascalCase: true }` and scope it via `overrides` to `*.tsx` files specifically, while keeping `kebabCase` for plain `.ts` utility files if the project mixes conventions by file type intentionally (documented, not accidental).

## See Also

- [name-PascalCase-types](name-PascalCase-types.md) - Use `PascalCase` for types, interfaces, classes, and enums
- [proj-feature-based-structure](proj-feature-based-structure.md) - Organize files by feature/domain rather than by technical layer
- [proj-colocate-tests](proj-colocate-tests.md) - Keep test files next to the source they test
