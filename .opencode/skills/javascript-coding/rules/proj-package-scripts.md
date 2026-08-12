# proj-package-scripts

> Use npm scripts for project tasks instead of shell scripts or global tools

## Why It Matters

npm scripts are cross-platform (they work on Windows, macOS, and Linux), self-documenting via `package.json`, and don't require developers to install global tools. They're discoverable via `npm run` and composable (scripts can call other scripts). Shell scripts (`build.sh`, `deploy.sh`) are OS-dependent and hidden from npm's tooling.

## Bad

```bash
# build.sh — OS-dependent, not discoverable
#!/bin/bash
rm -rf dist/
babel src/ -d dist/
cp -r public/ dist/
```

## Good

```jsonc
{
  "scripts": {
    "clean": "rm -rf dist/",
    "build": "npm run clean && babel src/ -d dist/ && cp -r public/ dist/",
    "dev": "node --watch src/server.js",
    "test": "node --test",
    "test:watch": "node --test --watch",
    "lint": "eslint .",
    "format": "prettier --write .",
    "start": "node dist/server.js",
    "db:migrate": "node scripts/migrate.js",
    "db:seed": "node scripts/seed.js"
  }
}
```

## Script Lifecycle Hooks

```jsonc
{
  "scripts": {
    "prebuild": "npm run lint && npm test",
    "build": "babel src/ -d dist/",
    "postbuild": "cp package.json dist/",
    "prepare": "husky || true"
  }
}
```

## When Exceptions Apply

Complex multi-step CI/CD pipelines may need dedicated shell scripts or CI configuration. Use npm scripts for development workflows and build steps.

## See Also

- [proj-lockfile-commit](./proj-lockfile-commit.md) - Commit lockfile
- [lint-husky-lint-staged](./lint-husky-lint-staged.md) - Pre-commit hooks
