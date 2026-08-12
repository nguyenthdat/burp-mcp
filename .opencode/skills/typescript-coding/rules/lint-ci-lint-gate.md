# lint-ci-lint-gate

> Run typecheck and lint as a required CI gate

## Why It Matters

A linter that runs only in a pre-commit hook or an editor extension is trivially bypassable — `git commit --no-verify`, a contributor without the hook installed, or an editor that silences warnings all let violations reach the shared branch. Worse, `tsc`'s type errors and ESLint's rule violations are two genuinely different checks (type errors don't run ESLint's parser rules, and vice versa), so both must run, both must be required (not just informational), and both must block the merge — otherwise "we have linting" is a claim without enforcement, and standards decay to whatever the least careful recent contributor did.

## Bad

```yaml
# .github/workflows/ci.yml — lint runs but isn't required, and typecheck is missing entirely
name: CI
on: [pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    continue-on-error: true # failures don't block merge
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npm run lint
```

## Good

```yaml
# .github/workflows/ci.yml
name: CI
on: [pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'
      - run: pnpm install --frozen-lockfile
      - run: pnpm run typecheck   # tsc --noEmit
      - run: pnpm run lint        # eslint .
      - run: pnpm run format:check # prettier --check .
      - run: pnpm run test
```

```json
// package.json
{
  "scripts": {
    "typecheck": "tsc --noEmit",
    "lint": "eslint .",
    "format:check": "prettier --check .",
    "test": "vitest run"
  }
}
```

Then mark the `verify` job as a required status check in the repository's branch protection rules, so a PR literally cannot be merged while any step fails — closing the loophole that `continue-on-error` or an unenforced check leaves open.

## Fail Fast, Fail Cheap

Order steps from cheapest/fastest to most expensive (typecheck and lint typically run in seconds; integration tests take longer) so a broken PR gets feedback quickly without waiting on the full suite.

## See Also

- [lint-typescript-eslint-recommended](lint-typescript-eslint-recommended.md) - Adopt `typescript-eslint`'s recommended (or recommended-type-checked) config
- [proj-lockfile-commit](proj-lockfile-commit.md) - Commit the lockfile for reproducible installs
- [test-coverage-meaningful](test-coverage-meaningful.md) - Track coverage that reflects real risk, not just percentage
