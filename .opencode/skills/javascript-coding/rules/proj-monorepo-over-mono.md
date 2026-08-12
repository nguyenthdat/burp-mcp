# proj-monorepo-over-mono

> Use npm/pnpm/yarn workspaces for multi-package projects instead of a monolithic structure

## Why It Matters

Monolithic projects grow unbounded: tests slow down, deployments couple unrelated changes, and dependency versions conflict. Workspaces split the codebase into independently versioned, tested, and deployed packages while sharing a single lockfile and `node_modules`. This enables parallel development, faster CI, and clear ownership boundaries.

## Bad

```jsonc
// Single giant package.json — everything mixed together
{
  "name": "my-app",
  "dependencies": {
    "express": "4.21.0",
    "react": "18.3.0",
    "pg": "8.12.0",
    "redis": "4.7.0",
    "bull": "4.16.0"
    // 100+ more dependencies — no separation of concerns
  }
}
```

## Good

```jsonc
// Root package.json — workspace configuration
{
  "name": "my-app",
  "private": true,
  "workspaces": [
    "packages/*",
    "apps/*"
  ],
  "scripts": {
    "dev": "npm run dev --workspaces",
    "test": "npm run test --workspaces",
    "build": "npm run build --workspaces"
  }
}

// packages/shared/package.json
{
  "name": "@my-app/shared",
  "version": "1.0.0",
  "main": "./dist/index.js"
}

// apps/api/package.json
{
  "name": "@my-app/api",
  "version": "1.0.0",
  "dependencies": {
    "@my-app/shared": "*",
    "express": "4.21.0"
  }
}

// apps/web/package.json
{
  "name": "@my-app/web",
  "version": "1.0.0",
  "dependencies": {
    "@my-app/shared": "*",
    "react": "18.3.0"
  }
}
```

## When Exceptions Apply

For projects with fewer than 3 independent deployable units, a single package is fine. Introduce workspaces when you need independent versioning, deployment, or code ownership.

## See Also

- [proj-src-lib-dir](./proj-src-lib-dir.md) - src/ and dist/ directory structure
- [mod-package-exports](./mod-package-exports.md) - Exports field
