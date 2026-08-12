# sec-dependency-audit

> Regularly run npm audit and pin dependency versions

## Why It Matters

Third-party dependencies are the most common source of security vulnerabilities in Node.js applications. A single unpatched dependency can expose your application to RCE, data exfiltration, or DoS. Regular auditing with `npm audit` and version pinning prevent supply-chain attacks and ensure reproducible builds.

## Bad

```jsonc
// package.json — loose version ranges
{
  "dependencies": {
    "express": "^4.18.0",     // ^ allows minor bumps — may include vulnerabilities
    "lodash": "*",             // * accepts any version — dangerous
    "axios": ">=1.0.0"         // >= allows potentially vulnerable future versions
  }
}
```

## Good

```jsonc
// package.json — pinned versions
{
  "dependencies": {
    "express": "4.21.0",
    "lodash": "4.17.21",
    "axios": "1.7.2"
  },
  "overrides": {
    // Force sub-dependency to use patched version
    "lodash": "4.17.21"
  },
  "scripts": {
    "audit": "npm audit",
    "audit:fix": "npm audit fix",
    "outdated": "npm outdated"
  }
}
```

## CI Audit Pipeline

```yaml
# .github/workflows/security.yml
name: Security Audit

on:
  schedule:
    - cron: '0 8 * * 1'  # Every Monday
  push:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
      - run: npm ci
      - run: npm audit --audit-level=high
```

## Lockfile Audit

```bash
# Check for known vulnerabilities
npm audit

# Fix auto-fixable vulnerabilities
npm audit fix

# Check for outdated packages
npm outdated

# Use socket.dev for supply-chain analysis
npx socket security
```

## When Exceptions Apply

In fast-moving development, `^` ranges are acceptable during active development with regular `npm update` runs. Pin versions before production releases.

## See Also

- [proj-lockfile-commit](./proj-lockfile-commit.md) - Always commit lockfile
- [proj-package-scripts](./proj-package-scripts.md) - npm scripts for tasks
