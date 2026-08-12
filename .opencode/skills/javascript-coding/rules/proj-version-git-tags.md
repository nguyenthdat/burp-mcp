# proj-version-git-tags

> Use git tags for releases — don't rely solely on the version field in package.json

## Why It Matters

The `version` field in `package.json` is updated manually and can become stale or incorrect. Git tags are immutable, cryptographically signable, and serve as the canonical source of truth for which commit constitutes a release. CI/CD pipelines, changelogs, and deployment triggers should use git tags, not package.json version.

## Bad

```bash
# Manually bumping package.json — easy to forget or get wrong
npm version patch  # Bumps package.json, but did you commit? Tag?
git add package.json
git commit -m "Bump to 1.2.4"
# Forgot to tag? Deployment doesn't know what's a release
```

## Good

```bash
# npm version creates both commit and tag
npm version patch  # Bumps package.json + creates git tag v1.2.4
git push --follow-tags

# CI/CD detects tag
# .github/workflows/release.yml
on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.ref_name }}
      - run: npm ci
      - run: npm run build
      - run: npm publish
```

## Tag Conventions

```bash
# Semantic versioning tags
git tag v1.0.0
git tag v1.0.1
git tag v1.1.0
git tag v2.0.0

# Pre-release tags
git tag v2.0.0-beta.1
git tag v2.0.0-rc.1

# Signed tags for security
git tag -s v1.0.0 -m "Release v1.0.0"
```

## When Exceptions Apply

Private monolithic applications that are deployed continuously from main may not need version tags. Tags are most valuable for libraries, services with versioned releases, and anything published to a registry.

## See Also

- [doc-changelog](./doc-changelog.md) - Maintain changelog
- [proj-nvm-rc](./proj-nvm-rc.md) - Node version management
