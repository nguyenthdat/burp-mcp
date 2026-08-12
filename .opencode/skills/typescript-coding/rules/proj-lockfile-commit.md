# proj-lockfile-commit

> Commit the lockfile for reproducible installs

## Why It Matters

`package.json` specifies version *ranges* (`^4.2.0`), not exact versions — without a lockfile, two installs run minutes apart can resolve different transitive dependency versions, silently changing behavior (or introducing a regression) between a developer's machine, CI, and production. The lockfile (`package-lock.json`, `pnpm-lock.yaml`, or `yarn.lock`) pins the exact resolved version and integrity hash of every dependency in the tree, so `npm ci` / `pnpm install --frozen-lockfile` reproduces the identical `node_modules` everywhere. Not committing it — or worse, having multiple lockfiles for different package managers in the same repo — reintroduces "works on my machine" nondeterminism that the lockfile exists specifically to eliminate.

## Bad

```
# .gitignore
node_modules/
package-lock.json   # ignored — every install can resolve different versions
```

```bash
# CI and local machines may install different transitive versions
# of the same declared range, causing "works locally, breaks in CI"
npm install
```

## Good

```
# .gitignore
node_modules/
# package-lock.json is committed, not ignored
```

```bash
# CI: fails fast if package.json and the lockfile have drifted,
# and installs the exact versions the lockfile specifies
npm ci

# pnpm equivalent
pnpm install --frozen-lockfile
```

## Avoid Multiple Lockfiles

Having `package-lock.json` and `pnpm-lock.yaml` both committed (e.g., because different contributors used different package managers locally) causes tools to disagree about the dependency tree and can silently install from the wrong one. Pick one package manager per repo and enforce it:

```jsonc
// package.json
{
  "packageManager": "pnpm@9.12.0",
  "engines": { "npm": "please-use-pnpm", "yarn": "please-use-pnpm" }
}
```

```bash
npx only-allow pnpm   # add as a "preinstall" script to hard-block the wrong CLI
```

## CI Usage

```yaml
# .github/workflows/ci.yml
- uses: actions/setup-node@v4
  with:
    node-version: 20
    cache: 'pnpm'
- run: pnpm install --frozen-lockfile
- run: pnpm run build
```

## See Also

- [proj-monorepo-workspaces](proj-monorepo-workspaces.md) - Use workspaces (pnpm/npm/yarn) to manage a monorepo's packages
- [lint-ci-lint-gate](lint-ci-lint-gate.md) - Run typecheck and lint as a required CI gate
- [proj-env-specific-config](proj-env-specific-config.md) - Keep environment-specific configuration separate from code
