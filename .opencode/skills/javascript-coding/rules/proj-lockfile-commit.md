# proj-lockfile-commit

> Always commit `package-lock.json` (or `yarn.lock`/`pnpm-lock.yaml`) to version control

## Why It Matters

The lockfile pins exact dependency versions, ensuring every developer, CI, and production deployment uses identical packages. Without a committed lockfile, `npm install` resolves versions differently over time, leading to "works on my machine" bugs and supply-chain attacks via compromised dependency updates. The lockfile IS the build manifest.

## Bad

```gitignore
# .gitignore — lockfile excluded
package-lock.json
```

```bash
# CI installs the latest patch versions — unpredictable
npm install  # Different than what developer tested
```

## Good

```gitignore
# .gitignore — lockfile committed
node_modules/
```

```bash
# CI uses exact locked versions
npm ci  # Installs exactly what's in package-lock.json
```

## CI Must Use npm ci

```yaml
# .github/workflows/test.yml
- run: npm ci        # Respects lockfile, fails if out of sync
- run: npm test
```

## When Exceptions Apply

Libraries published to npm should NOT commit the lockfile (add `package-lock.json` to `.gitignore`). Applications and services MUST commit it.

## See Also

- [sec-dependency-audit](./sec-dependency-audit.md) - Audit dependencies
- [proj-package-scripts](./proj-package-scripts.md) - npm scripts
