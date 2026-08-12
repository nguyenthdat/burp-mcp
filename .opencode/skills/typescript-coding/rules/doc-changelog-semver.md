# doc-changelog-semver

> Maintain a CHANGELOG that follows semantic versioning

## Why It Matters

Without a changelog, consumers upgrading a dependency have no way to know whether a version bump is safe to take automatically or requires code changes, so they either pin versions forever (missing bug fixes and security patches) or upgrade blind and get broken by an undocumented breaking change. Pairing a CHANGELOG with strict semantic versioning (major.minor.patch) turns the version number itself into a signal — patch releases are always safe, minor releases add functionality without breaking anything, major releases may require migration — and the changelog explains exactly what changed for each.

## Bad

```markdown
# Changelog

- fixed stuff
- new feature
- v2.1.4
```

## Good

```markdown
# Changelog

All notable changes to this project are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/).

## [2.1.0] - 2026-07-18

### Added
- `RateLimiter.tryConsumeMany(key, count)` for consuming multiple tokens atomically.

### Fixed
- Fixed a race condition where concurrent `tryConsume` calls on the Redis backend
  could over-consume tokens under high concurrency (#142).

## [2.0.0] - 2026-06-02

### Changed
- **Breaking:** `RateLimiter.tryConsume` now returns `Promise<boolean>` instead of
  `boolean`, since the Redis backend requires an async round-trip. See
  MIGRATION.md#v2 for the upgrade path.
```

## Semver Rules Cheat Sheet

| Change | Version bump | Example |
|---|---|---|
| Bug fix, no API change | Patch (`2.1.3` → `2.1.4`) | Fixed a race condition |
| New backward-compatible feature | Minor (`2.1.4` → `2.2.0`) | Added a new optional parameter |
| Breaking change to public API | Major (`2.2.0` → `3.0.0`) | Changed a return type, removed an export |

Use `changesets` or `semantic-release` to generate the CHANGELOG from commit messages or PR-level change descriptions automatically, so it can't drift out of sync with actual releases. Every entry should be understandable by a consumer who never read the PR — link to migration docs for breaking changes instead of assuming context.

## See Also

- [doc-deprecated-tag](doc-deprecated-tag.md) - Mark deprecated APIs with `@deprecated` and a migration path
- [api-versioned-public-api](api-versioned-public-api.md) - version a public API deliberately across breaking changes
- [doc-readme-package](doc-readme-package.md) - Maintain a README with install/usage examples for every published package
