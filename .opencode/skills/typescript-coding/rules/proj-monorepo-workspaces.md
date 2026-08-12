# proj-monorepo-workspaces

> Use workspaces (pnpm/npm/yarn) to manage a monorepo's packages

## Why It Matters

Without workspaces, a multi-package repository either duplicates `node_modules` per package (wasting disk space and install time) or relies on manual `npm link`/relative `file:` references that drift out of sync and don't resolve devDependencies or scripts consistently. Workspaces let a package manager understand that `packages/api` and `packages/ui` live in the same repo, hoist shared dependencies into a single install, symlink internal packages together automatically, and let you run a script across every package with one command (`pnpm -r build`). This is the difference between "a folder of unrelated projects" and an actual monorepo with atomic cross-package changes and a single lockfile.

## Bad

```jsonc
// packages/ui/package.json — manually pointing at a local path
{
  "dependencies": {
    "@myorg/shared": "file:../shared" // easy to forget to rebuild/relink
  }
}
```

```
repo/
  packages/
    api/node_modules/    (full copy of every dependency)
    ui/node_modules/     (another full copy, possibly different versions)
    shared/node_modules/ (a third copy)
```

## Good

```yaml
# pnpm-workspace.yaml
packages:
  - "packages/*"
  - "apps/*"
```

```jsonc
// packages/ui/package.json
{
  "name": "@myorg/ui",
  "dependencies": {
    "@myorg/shared": "workspace:*" // resolved to the local package, always in sync
  }
}
```

```bash
pnpm install                 # single install, hoisted node_modules, one lockfile
pnpm -r build                # run "build" script in every package, in dependency order
pnpm --filter @myorg/ui dev  # run a script in just one package
```

## Comparison

| Manager | Workspace syntax | Notes |
|---|---|---|
| pnpm | `pnpm-workspace.yaml` + `workspace:*` | Strict, non-flat `node_modules` by default; fastest installs; catches phantom dependencies |
| npm | `"workspaces"` array in root `package.json` | Built into npm since v7; flat `node_modules` |
| Yarn (Berry) | `"workspaces"` array + `.yarnrc.yml` | Plug'n'Play mode changes resolution; supports `workspace:*` too |

Combine workspaces with a task runner (Turborepo, Nx) once cross-package build caching and dependency-ordered task execution become a bottleneck — workspaces alone only handle dependency linking and installs, not build orchestration or caching.

## See Also

- [proj-lockfile-commit](proj-lockfile-commit.md) - Commit the lockfile for reproducible installs
- [proj-single-tsconfig-base](proj-single-tsconfig-base.md) - Share a base `tsconfig.json` and extend it per package
- [proj-module-boundaries](proj-module-boundaries.md) - Enforce module boundaries; don't import another module's internal files
