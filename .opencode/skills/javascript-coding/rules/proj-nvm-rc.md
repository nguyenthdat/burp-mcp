# proj-nvm-rc

> Include `.nvmrc` or `engines` field to document the required Node.js version

## Why It Matters

Different Node.js versions have different APIs, performance characteristics, and bugs. Without a documented version, developers use whatever they have installed — leading to "works on my machine" issues when the CI or production runs a different version. `.nvmrc` enables `nvm use` to automatically switch, and `engines` warns on mismatched versions.

## Bad

```jsonc
// No version indication — developers guess
{
  "name": "my-app"
}
```

## Good

```bash
# .nvmrc — used by nvm, fnm, volta
22
```

```jsonc
// package.json — warns on npm install if version mismatch
{
  "name": "my-app",
  "engines": {
    "node": ">=20.0.0"
  }
}
```

## Enforce in CI

```jsonc
// .npmrc — fail on engine mismatch
engine-strict=true
```

```yaml
# .github/workflows/test.yml
jobs:
  test:
    strategy:
      matrix:
        node-version: [20, 22]
    steps:
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}
      - run: npm ci
      - run: npm test
```

## When Exceptions Apply

Libraries targeting a wide range of Node.js versions should set `engines` to the minimum supported version. Applications should be specific.

## See Also

- [proj-version-git-tags](./proj-version-git-tags.md) - Release versioning
- [mod-esm-over-cjs](./mod-esm-over-cjs.md) - ESM requires Node 12+
