# doc-readme-package

> Maintain a README with install/usage examples for every published package

## Why It Matters

A published package without a README forces every potential consumer to read source code or type declarations just to figure out how to install and call it, which is friction most people won't pay — they'll pick a competing package instead or, worse, guess wrong and file bug reports for intended behavior. A README with install instructions and a minimal working example is the difference between a package people can adopt in five minutes and one that sits unused regardless of how good the underlying code is.

## Bad

```markdown
# my-package

A package.
```

## Good

```markdown
# @acme/rate-limiter

Token-bucket rate limiting for Node.js, with pluggable storage backends
(in-memory, Redis).

## Install

\`\`\`bash
npm install @acme/rate-limiter
\`\`\`

## Usage

\`\`\`typescript
import { RateLimiter } from "@acme/rate-limiter";

const limiter = new RateLimiter({ capacity: 10, refillPerSecond: 1 });

if (await limiter.tryConsume("user:123")) {
  // proceed with the request
} else {
  // reject with 429
}
\`\`\`

## API

See the [full API reference](https://acme.dev/docs/rate-limiter).

## License

MIT
```

## What Every Package README Needs

| Section | Purpose |
|---|---|
| One-line description | What the package does and why it exists, right under the title |
| Install | The exact `npm`/`pnpm`/`yarn` install command |
| Usage | A minimal, copy-pasteable example that actually runs |
| Requirements | Node/TypeScript version support, peer dependencies |
| API reference or link | Full signature docs, either inline or linked to generated `typedoc` output |
| License | Especially important for open-source packages |

Keep the top-level usage example in sync with the actual public API — a stale README example that no longer type-checks is worse than none, since it actively misleads. Consider a `README` test (a doctest-style script that extracts and runs fenced code blocks) for packages where this drifts often.

## See Also

- [doc-example-tags](doc-example-tags.md) - Include an `@example` block in non-trivial doc comments
- [doc-changelog-semver](doc-changelog-semver.md) - Maintain a CHANGELOG that follows semantic versioning
- [api-minimal-surface](api-minimal-surface.md) - keep the public API small enough to document well
