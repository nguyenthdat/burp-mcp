# doc-changelog

> Maintain a CHANGELOG.md following semantic versioning conventions

## Why It Matters

Consumers need to know what changed between versions to decide whether to upgrade and what migration work is needed. A changelog documents breaking changes, new features, and bug fixes in a human-readable format. It's required by many compliance frameworks and is a signal of project maturity.

## Bad

```markdown
# Changelog

## 2.0.0
- Fixed some bugs
- Added new features
- Performance improvements
```

## Good

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2024-06-15

### Added
- `cache.setTTL(key, ttlMs)` for per-key expiry (#42)
- Support for Redis Cluster connections (#45)

### Fixed
- Memory leak in long-running subscription handlers (#38)
- Race condition in `cache.getOrSet()` under high concurrency (#41)

### Deprecated
- `cache.expire(key, seconds)` — use `cache.setTTL(key, ms)` instead

## [2.0.0] - 2024-05-01

### Breaking
- Minimum Node.js version is now 20.0.0
- `createClient()` now returns a Promise instead of a Client
- Removed `legacyConnect()` — use `createClient()` instead
- Configuration format changed: `host`/`port` moved to `connection` object

### Added
- Automatic reconnection with exponential backoff

### Migration Guide

```js
// Before (v1.x)
const client = createClient({ host: 'localhost', port: 6379 });

// After (v2.x)
const client = await createClient({
  connection: { host: 'localhost', port: 6379 },
});
```
```

## When Exceptions Apply

Internal tools and unreleased projects don't need a changelog. Start maintaining one before the first public release.

## See Also

- [doc-readme-essentials](./doc-readme-essentials.md) - README essentials
- [proj-version-git-tags](./proj-version-git-tags.md) - Git tags for releases
