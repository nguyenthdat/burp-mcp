---
name: javascript-coding
description: "Comprehensive idiomatic JavaScript (Node.js/ES2024+) guidance: 181 prioritized rules across 14 categories. Use when writing, reviewing, refactoring, optimizing, or debugging JavaScript (`.js`, `.mjs`, `.cjs`, `package.json`). Covers async/await, error handling, modern ES2024+ features, Node.js APIs, security, testing, performance patterns, and anti-patterns. Preserve the target project's Node.js version and toolchain constraints."
compatibility: opencode
metadata:
  domain: javascript
  audience: software-engineer
  edition: es2024-node
---

# JavaScript Best Practices

Comprehensive guide for writing high-quality, idiomatic JavaScript (ES2024+, Node.js 20+). Contains 181 rules across 14 categories, prioritized by impact to guide LLMs in code generation and refactoring.

## When to Apply

Reference these guidelines when:
- Writing new JavaScript modules, classes, or functions
- Implementing async operations or error handling
- Designing public APIs for libraries and applications
- Reviewing code for correctness and maintainability
- Optimizing performance or reducing memory pressure
- Setting up project structure, linting, or CI pipelines
- Refactoring existing JavaScript code
- Migrating from CommonJS to ES modules

## Modern JavaScript & ES2024+ Notes

This skill targets **Node.js 20+** with ES2024 features. Use `"type": "module"` in `package.json` and declare your minimum Node version:

```json
{
  "type": "module",
  "engines": { "node": ">=20.0.0" }
}
```

Key modern features covered throughout these rules:

- **ESM vs CJS.** Prefer ES modules (`import`/`export`) over CommonJS (`require`/`module.exports`). Node.js 20+ has stable ESM support. Use `.mjs` extension or `"type": "module"` in package.json.
- **Top-level await.** Available in ES modules without an async wrapper. Use in entry points and scripts, not in libraries (blocks consumers).
- **Private class fields.** Use `#privateField` and `#privateMethod()` for true encapsulation — not `_pseudoPrivate` conventions.
- **Array/Set/Map improvements.** `Array.prototype.toSorted()`, `.toReversed()`, `.toSpliced()`, `.with()` — immutable array methods. `Set.prototype.union()`, `.intersection()`, `.difference()`.
- **Temporal API.** `Temporal.Now.plainDateISO()`, `Temporal.Duration`, `Temporal.ZonedDateTime` replace legacy `Date`. Available as a polyfill; Stage 3 in TC39.
- **`Error.cause`.** Chain errors with `new Error('message', { cause: originalError })` for rich error traces.
- **`structuredClone()`.** Deep-clone objects with circular references, Date, Map, Set, ArrayBuffer — replaces `JSON.parse(JSON.stringify())`.
- **`Promise.withResolvers()`.** `const { promise, resolve, reject } = Promise.withResolvers()` — cleaner than the Promise constructor pattern.
- **`Array.fromAsync()`.** Convert async iterables to arrays: `await Array.fromAsync(asyncIterable)`.
- **Pipeline operator.** `|> ` is Stage 2 (not yet in ES2024). Use in projects with Babel/TypeScript transpilation.
- **Decorators.** TC39 decorators are Stage 3. Use the 2023-11 proposal syntax, not the legacy TypeScript experimental syntax.
- **Import attributes.** `import config from './config.json' with { type: 'json' }` — secure JSON imports with explicit type assertions.

## Rule Categories by Priority

| Priority | Category | Impact | Prefix | Rules |
|----------|----------|--------|--------|-------|
| 1 | Async/Await & Concurrency | CRITICAL | `async-` | 16 |
| 2 | Error Handling | CRITICAL | `err-` | 14 |
| 3 | Security | CRITICAL | `sec-` | 14 |
| 4 | Module Design & Exports | HIGH | `mod-` | 12 |
| 5 | Functional & Data Patterns | HIGH | `fn-` | 14 |
| 6 | Type Safety & Validation (vanilla JS) | HIGH | `type-` | 10 |
| 7 | Naming Conventions | MEDIUM | `name-` | 14 |
| 8 | Testing | MEDIUM | `test-` | 13 |
| 9 | Documentation (JSDoc) | MEDIUM | `doc-` | 11 |
| 10 | Node.js/Runtime | MEDIUM | `node-` | 14 |
| 11 | Performance & Memory | MEDIUM | `perf-` | 12 |
| 12 | Project Structure & Tooling | LOW | `proj-` | 11 |
| 13 | Linting (ESLint) | LOW | `lint-` | 11 |
| 14 | Anti-patterns | REFERENCE | `anti-` | 15 |

---

## Quick Reference

### 1. Async/Await & Concurrency (CRITICAL)

- [`async-avoid-callback-hell`](rules/async-avoid-callback-hell.md) - Prefer async/await over nested callbacks
- [`async-no-blocking-loop`](rules/async-no-blocking-loop.md) - Don't use sync fs/math/crypto in async contexts
- [`async-return-promise`](rules/async-return-promise.md) - Always return promises or await in async functions
- [`async-parallel-over-sequential`](rules/async-parallel-over-sequential.md) - Use Promise.all/map concurrency over sequential awaits
- [`async-promise-allsettled`](rules/async-promise-allsettled.md) - Use Promise.allSettled when partial failures are ok
- [`async-error-swallowed`](rules/async-error-swallowed.md) - Don't forget .catch() — handle promise rejections
- [`async-top-level-react`](rules/async-top-level-react.md) - Use top-level await in ES modules (Node 14.8+)
- [`async-abort-control`](rules/async-abort-control.md) - Use AbortController for cancellable async operations
- [`async-timeout-pattern`](rules/async-timeout-pattern.md) - Wrap async operations with timeout
- [`async-retry-pattern`](rules/async-retry-pattern.md) - Implement exponential backoff for transient failures
- [`async-avoid-for-await-misuse`](rules/async-avoid-for-await-misuse.md) - Don't use for-await on non-async iterables
- [`async-microtask-queue`](rules/async-microtask-queue.md) - Understand microtask vs macrotask ordering
- [`async-avoid-floating-promises`](rules/async-avoid-floating-promises.md) - Never create promises without handling their resolution
- [`async-promise-with-resolvers`](rules/async-promise-with-resolvers.md) - Use Promise.withResolvers() (ES2024) instead of Promise constructor
- [`async-array-from-async`](rules/async-array-from-async.md) - Use Array.fromAsync() (ES2024) over manual async mapping
- [`async-worker-threads`](rules/async-worker-threads.md) - Use worker_threads for CPU-intensive work

### 2. Error Handling (CRITICAL)

- [`err-custom-error-classes`](rules/err-custom-error-classes.md) - Extend Error class for domain errors
- [`err-error-cause`](rules/err-error-cause.md) - Use `{ cause: err }` (ES2022) for error chaining
- [`err-no-throw-string`](rules/err-no-throw-string.md) - Always throw Error instances, never strings
- [`err-avoid-silent-catch`](rules/err-avoid-silent-catch.md) - Always log or rethrow caught errors
- [`err-structured-error-handling`](rules/err-structured-error-handling.md) - Create error hierarchy with instanceof checks
- [`err-aggregate-error`](rules/err-aggregate-error.md) - Use AggregateError for multiple errors
- [`err-assertion-libraries`](rules/err-assertion-libraries.md) - Use assertion for input validation with assertion libraries
- [`err-type-check-inputs`](rules/err-type-check-inputs.md) - Validate types at function boundaries
- [`err-try-catch-narrow`](rules/err-try-catch-narrow.md) - Keep try blocks as small as possible
- [`err-async-await-catch`](rules/err-async-await-catch.md) - Use try/catch with await, not .catch() chains
- [`err-finally-cleanup`](rules/err-finally-cleanup.md) - Use finally blocks for cleanup
- [`err-no-swallow-rejection`](rules/err-no-swallow-rejection.md) - Don't swallow errors in catch blocks without handling
- [`err-error-info`](rules/err-error-info.md) - Attach useful info (code, statusCode) to custom errors
- [`err-global-handlers`](rules/err-global-handlers.md) - Set process.on('uncaughtException') and unhandledRejection

### 3. Security (CRITICAL)

- [`sec-no-eval`](rules/sec-no-eval.md) - Never use eval() or Function() constructor with user input
- [`sec-input-sanitize`](rules/sec-input-sanitize.md) - Always sanitize user input before use
- [`sec-avoid-os-command`](rules/sec-avoid-os-command.md) - Avoid child_process.exec(); prefer execFile or spawn
- [`sec-sql-injection`](rules/sec-sql-injection.md) - Use parameterized queries, never string interpolation
- [`sec-path-traversal`](rules/sec-path-traversal.md) - Validate file paths — reject `../` escapes
- [`sec-no-hardcoded-secrets`](rules/sec-no-hardcoded-secrets.md) - Never commit secrets; use environment variables
- [`sec-helmet-headers`](rules/sec-helmet-headers.md) - Set security headers (Helmet or manual)
- [`sec-csrf-protection`](rules/sec-csrf-protection.md) - Use CSRF tokens for state-changing operations
- [`sec-rate-limit`](rules/sec-rate-limit.md) - Implement rate limiting on endpoints
- [`sec-input-size-limits`](rules/sec-input-size-limits.md) - Limit size of incoming data streams
- [`sec-dependency-audit`](rules/sec-dependency-audit.md) - Regularly run npm audit; pin versions
- [`sec-prototype-pollution`](rules/sec-prototype-pollution.md) - Guard against prototype pollution
- [`sec-samesite-cookies`](rules/sec-samesite-cookies.md) - Set SameSite=Strict or Lax on cookies
- [`sec-regex-dos`](rules/sec-regex-dos.md) - Avoid catastrophic backtracking in regex with untrusted input

### 4. Module Design & Exports (HIGH)

- [`mod-esm-over-cjs`](rules/mod-esm-over-cjs.md) - Prefer ES modules over CommonJS
- [`mod-named-over-default`](rules/mod-named-over-default.md) - Prefer named exports over default exports
- [`mod-barrel-files`](rules/mod-barrel-files.md) - Use index.js barrel files sparingly (tree-shaking)
- [`mod-no-mixed-modules`](rules/mod-no-mixed-modules.md) - Don't mix import and require in the same file
- [`mod-side-effect-free`](rules/mod-side-effect-free.md) - Keep modules side-effect-free; reserve side effects for entry points
- [`mod-circular-deps`](rules/mod-circular-deps.md) - Avoid circular dependencies
- [`mod-export-near-definition`](rules/mod-export-near-definition.md) - Export functions close to where they're defined
- [`mod-import-order`](rules/mod-import-order.md) - Organize imports: node builtins → npm packages → local modules
- [`mod-package-exports`](rules/mod-package-exports.md) - Use exports field in package.json
- [`mod-separate-concerns`](rules/mod-separate-concerns.md) - One module, one responsibility
- [`mod-no-barrel-re-export-star`](rules/mod-no-barrel-re-export-star.md) - Avoid `export * from` in library code
- [`mod-dynamic-import`](rules/mod-dynamic-import.md) - Use dynamic import() for lazy loading large modules

### 5. Functional & Data Patterns (HIGH)

- [`fn-immutability`](rules/fn-immutability.md) - Prefer const, spread, and Object.freeze over mutation
- [`fn-pure-functions`](rules/fn-pure-functions.md) - Write pure functions that don't mutate inputs
- [`fn-map-over-for`](rules/fn-map-over-for.md) - Prefer .map()/.filter()/.reduce() over for loops
- [`fn-optional-chaining`](rules/fn-optional-chaining.md) - Use `?.` and `??` for safe property access
- [`fn-nullish-coalescing`](rules/fn-nullish-coalescing.md) - Prefer `??` over `||` for default values
- [`fn-destructure`](rules/fn-destructure.md) - Use destructuring for function parameters and object access
- [`fn-template-literals`](rules/fn-template-literals.md) - Use template literals over string concatenation
- [`fn-default-params`](rules/fn-default-params.md) - Use default function parameters over manual checks
- [`fn-rest-spread`](rules/fn-rest-spread.md) - Use rest/spread operators over arguments object
- [`fn-avoid-delete-mutation`](rules/fn-avoid-delete-mutation.md) - Avoid `delete` on objects (use `undefined` or omit)
- [`fn-composition-over-inheritance`](rules/fn-composition-over-inheritance.md) - Prefer function composition over class inheritance
- [`fn-structured-clone`](rules/fn-structured-clone.md) - Use structuredClone() over JSON.parse(JSON.stringify())
- [`fn-groupBy-toMap`](rules/fn-groupBy-toMap.md) - Use Map.groupBy() (ES2024) for grouping
- [`fn-temporal-over-date`](rules/fn-temporal-over-date.md) - Use Temporal API (ES2024) over legacy Date

### 6. Type Safety & Validation (vanilla JS) (HIGH)

- [`type-typeof-guards`](rules/type-typeof-guards.md) - Use typeof/instanceof checks at public boundaries
- [`type-zod-validation`](rules/type-zod-validation.md) - Use validation libraries (zod, joi) for API input
- [`type-no-magic-strings`](rules/type-no-magic-strings.md) - Use constants/enums over magic strings
- [`type-null-over-undefined`](rules/type-null-over-undefined.md) - Prefer null for intentional absence, undefined for missing
- [`type-avoid-implicit-coercion`](rules/type-avoid-implicit-coercion.md) - Use === !== over == !=
- [`type-validate-config`](rules/type-validate-config.md) - Validate environment/config at startup
- [`type-parse-dont-assume`](rules/type-parse-dont-assume.md) - Parse JSON/API responses with schema validation
- [`type-symbol-over-string`](rules/type-symbol-over-string.md) - Use Symbol for truly unique property keys
- [`type-tagged-unions`](rules/type-tagged-unions.md) - Use discriminated unions with a `type` field
- [`type-bigint-precision`](rules/type-bigint-precision.md) - Use BigInt for large numbers beyond Number.MAX_SAFE_INTEGER

### 7. Naming Conventions (MEDIUM)

- [`name-camelCase`](rules/name-camelCase.md) - Use camelCase for variables, functions, methods
- [`name-PascalCase`](rules/name-PascalCase.md) - Use PascalCase for classes and constructor functions
- [`name-UPPER_SNAKE`](rules/name-UPPER_SNAKE.md) - Use UPPER_SNAKE_CASE for constants
- [`name-kebab-case`](rules/name-kebab-case.md) - Use kebab-case for file names
- [`name-is-has-bool`](rules/name-is-has-bool.md) - Prefix booleans with is/has/should/can
- [`name-handler-verbs`](rules/name-handler-verbs.md) - Use on/handle prefix for event handlers
- [`name-verb-function`](rules/name-verb-function.md) - Start functions with a verb (getUser, not user)
- [`name-no-abbrev`](rules/name-no-abbrev.md) - Avoid abbreviations except widely known (req/res/err)
- [`name-collection-plural`](rules/name-collection-plural.md) - Use plural names for arrays/collections
- [`name-callback-descriptive`](rules/name-callback-descriptive.md) - Name callbacks descriptively, not cb/fn
- [`name-avoid-single-letter`](rules/name-avoid-single-letter.md) - Avoid single-letter names except loop indices (i, j)
- [`name-test-descriptive`](rules/name-test-descriptive.md) - Use should/when format for test names
- [`name-middleware-suffix`](rules/name-middleware-suffix.md) - Suffix middleware functions with Middleware
- [`name-private-underscore`](rules/name-private-underscore.md) - Use `_` prefix for module-private exports

### 8. Testing (MEDIUM)

- [`test-node-test-runner`](rules/test-node-test-runner.md) - Use Node.js built-in test runner (node --test)
- [`test-arrange-act-assert`](rules/test-arrange-act-assert.md) - Structure tests: arrange, act, assert
- [`test-describe-it`](rules/test-describe-it.md) - Group tests with describe/it blocks
- [`test-mock-over-stub`](rules/test-mock-over-stub.md) - Use mocks for behaviors, not stubs for implementation details
- [`test-isolation`](rules/test-isolation.md) - Tests must not depend on each other or shared mutable state
- [`test-async-testing`](rules/test-async-testing.md) - Always await or return promises in async tests
- [`test-setup-teardown`](rules/test-setup-teardown.md) - Use beforeEach/afterEach for setup and cleanup
- [`test-snapshot-cautious`](rules/test-snapshot-cautious.md) - Use snapshots sparingly; review diffs carefully
- [`test-coverage-threshold`](rules/test-coverage-threshold.md) - Set minimum coverage thresholds in CI
- [`test-fast-parallel`](rules/test-fast-parallel.md) - Keep tests fast; run in parallel when possible
- [`test-no-only`](rules/test-no-only.md) - Never commit .only() or .skip() to main branch
- [`test-integration-separate`](rules/test-integration-separate.md) - Separate unit tests from integration tests
- [`test-fixture-factories`](rules/test-fixture-factories.md) - Use factory functions for test data, not hardcoded fixtures

### 9. Documentation / JSDoc (MEDIUM)

- [`doc-jsdoc-public`](rules/doc-jsdoc-public.md) - Use JSDoc for all public API functions
- [`doc-param-return`](rules/doc-param-return.md) - Document @param and @returns in JSDoc
- [`doc-examples-in-jsdoc`](rules/doc-examples-in-jsdoc.md) - Include @example in complex function docs
- [`doc-type-annotations`](rules/doc-type-annotations.md) - Use @type, @typedef for plain JS type documentation
- [`doc-deprecated-tag`](rules/doc-deprecated-tag.md) - Use @deprecated with migration guidance
- [`doc-readme-essentials`](rules/doc-readme-essentials.md) - Include install, usage, API, license in README
- [`doc-changelog`](rules/doc-changelog.md) - Maintain CHANGELOG.md with semantic versioning
- [`doc-inline-comments-why`](rules/doc-inline-comments-why.md) - Comment WHY not WHAT
- [`doc-no-stale-comments`](rules/doc-no-stale-comments.md) - Remove commented-out code; use git history
- [`doc-private-internal`](rules/doc-private-internal.md) - Use @private/@internal for non-public APIs
- [`doc-see-references`](rules/doc-see-references.md) - Use @see for cross-references

### 10. Node.js / Runtime (MEDIUM)

- [`node-fs-async-over-sync`](rules/node-fs-async-over-sync.md) - Use fs/promises over fs sync methods
- [`node-stream-over-buffer`](rules/node-stream-over-buffer.md) - Use streams for large data; avoid loading into memory
- [`node-process-exit`](rules/node-process-exit.md) - Use process.exit() codes consistently
- [`node-env-config`](rules/node-env-config.md) - Load env from a single config module
- [`node-event-emitter-memory`](rules/node-event-emitter-memory.md) - Remove listeners to prevent memory leaks
- [`node-cluster-fork`](rules/node-cluster-fork.md) - Use cluster or pm2 for multi-core utilization
- [`node-signal-handling`](rules/node-signal-handling.md) - Handle SIGTERM/SIGINT for graceful shutdown
- [`node-avoid-sync-require`](rules/node-avoid-sync-require.md) - Don't require() dynamically inside async functions
- [`node-path-join`](rules/node-path-join.md) - Use path.join() over string concatenation
- [`node-url-over-strings`](rules/node-url-over-strings.md) - Use URL constructor over string URL manipulation
- [`node-buffer-deprecation`](rules/node-buffer-deprecation.md) - Use Buffer.from(), not new Buffer()
- [`node-http-agent`](rules/node-http-agent.md) - Reuse HTTP agents for connection pooling
- [`node-child-safe`](rules/node-child-safe.md) - Prefer spawn over exec; never exec with user input
- [`node-esm-migration`](rules/node-esm-migration.md) - Use import.meta.url instead of __dirname

### 11. Performance & Memory (MEDIUM)

- [`perf-debounce-throttle`](rules/perf-debounce-throttle.md) - Use debounce/throttle for frequent events
- [`perf-lazy-load`](rules/perf-lazy-load.md) - Lazy-load heavy modules with dynamic import()
- [`perf-avoid-sync-fs`](rules/perf-avoid-sync-fs.md) - Never use sync fs/path operations in request handlers
- [`perf-object-pool`](rules/perf-object-pool.md) - Use object pooling for frequent GC pressure
- [`perf-for-of-array`](rules/perf-for-of-array.md) - for...of is faster than forEach for arrays
- [`perf-set-over-array-lookup`](rules/perf-set-over-array-lookup.md) - Use Set/Map for O(1) lookups over array.includes
- [`perf-prepare-regex`](rules/perf-prepare-regex.md) - Compile regex outside loops
- [`perf-array-push-spread`](rules/perf-array-push-spread.md) - Use .push(...items) over .concat() for large arrays
- [`perf-json-parse-stream`](rules/perf-json-parse-stream.md) - Use streaming JSON parser for large payloads
- [`perf-avoid-string-concat-loops`](rules/perf-avoid-string-concat-loops.md) - Use array.join() over string += in loops
- [`perf-number-over-parseInt`](rules/perf-number-over-parseInt.md) - Use Number() or + over parseInt for coercion
- [`perf-memoize`](rules/perf-memoize.md) - Memoize expensive pure functions

### 12. Project Structure & Tooling (LOW)

- [`proj-monorepo-over-mono`](rules/proj-monorepo-over-mono.md) - Use workspaces for multi-package projects
- [`proj-src-lib-dir`](rules/proj-src-lib-dir.md) - Use src/ for source, dist/ for build output
- [`proj-env-files`](rules/proj-env-files.md) - Use .env.example not .env in git
- [`proj-package-scripts`](rules/proj-package-scripts.md) - Use npm scripts over shell scripts for tasks
- [`proj-lockfile-commit`](rules/proj-lockfile-commit.md) - Always commit package-lock.json
- [`proj-separate-configs`](rules/proj-separate-configs.md) - Separate config from code; use config/ or .env
- [`proj-middleware-stack`](rules/proj-middleware-stack.md) - Compose middleware in a dedicated file
- [`proj-layer-architecture`](rules/proj-layer-architecture.md) - Follow controller → service → repository layers
- [`proj-no-giant-files`](rules/proj-no-giant-files.md) - Keep files under 300 lines
- [`proj-version-git-tags`](rules/proj-version-git-tags.md) - Use git tags for releases, not package.json version only
- [`proj-nvm-rc`](rules/proj-nvm-rc.md) - Include .nvmrc or engines field

### 13. Linting / ESLint (LOW)

- [`lint-eslint-setup`](rules/lint-eslint-setup.md) - Use ESLint with flat config (eslint.config.mjs)
- [`lint-no-console-prod`](rules/lint-no-console-prod.md) - Warn on console.log in production code
- [`lint-no-var`](rules/lint-no-var.md) - Use let/const, never var
- [`lint-prefer-const`](rules/lint-prefer-const.md) - Use const by default; let only for reassignment
- [`lint-strict-comparisons`](rules/lint-strict-comparisons.md) - Enforce === and !==
- [`lint-no-param-reassign`](rules/lint-no-param-reassign.md) - Don't reassign function parameters
- [`lint-max-params`](rules/lint-max-params.md) - Limit function parameters to 3-4 max
- [`lint-complexity-limit`](rules/lint-complexity-limit.md) - Limit cyclomatic complexity per function
- [`lint-unused-vars-error`](rules/lint-unused-vars-error.md) - Treat unused variables as errors
- [`lint-prettier-format`](rules/lint-prettier-format.md) - Use Prettier for consistent formatting
- [`lint-husky-lint-staged`](rules/lint-husky-lint-staged.md) - Use lint-staged + husky for pre-commit checks

### 14. Anti-patterns (REFERENCE)

- [`anti-callback-hell`](rules/anti-callback-hell.md) - Don't nest callbacks deeply
- [`anti-global-mutation`](rules/anti-global-mutation.md) - Don't mutate global objects (Array.prototype, etc.)
- [`anti-sync-in-async`](rules/anti-sync-in-async.md) - Don't use sync I/O in async request handlers
- [`anti-await-loop`](rules/anti-await-loop.md) - Don't use await inside forEach/map (use for-of or Promise.all)
- [`anti-delete-operator`](rules/anti-delete-operator.md) - Don't use delete on arrays or objects in hot paths
- [`anti-nested-ternary`](rules/anti-nested-ternary.md) - Don't nest ternary operators
- [`anti-overly-smart-code`](rules/anti-overly-smart-code.md) - Don't sacrifice readability for cleverness
- [`anti-new-object-boolean`](rules/anti-new-object-boolean.md) - Don't use new Boolean/String/Number
- [`anti-arguments-object`](rules/anti-arguments-object.md) - Don't use `arguments` in non-arrow functions; use rest params
- [`anti-constructor-return`](rules/anti-constructor-return.md) - Don't return from a constructor
- [`anti-dynamic-export`](rules/anti-dynamic-export.md) - Don't use export default conditionally
- [`anti-redundant-await`](rules/anti-redundant-await.md) - Don't await non-promise values unnecessarily
- [`anti-parseint-no-radix`](rules/anti-parseint-no-radix.md) - Always pass radix to parseInt()
- [`anti-in-operator-array`](rules/anti-in-operator-array.md) - Don't use `in` to iterate arrays
- [`anti-throw-non-error`](rules/anti-throw-non-error.md) - Don't throw non-Error objects

---

## Recommended Config

```jsonc
// package.json settings for quality JS
{
  "type": "module",
  "engines": { "node": ">=20.0.0" },
  "scripts": {
    "lint": "eslint .",
    "test": "node --test",
    "format": "prettier --write .",
    "typecheck": "npx --yes typescript --noEmit --allowJs --checkJs .",
    "prepare": "husky || true"
  }
}
```

```js
// eslint.config.mjs (flat config)
import js from "@eslint/js";

export default [
  js.configs.recommended,
  {
    rules: {
      "no-var": "error",
      "prefer-const": "error",
      "eqeqeq": ["error", "always"],
      "no-unused-vars": ["error", { "argsIgnorePattern": "^_" }],
      "no-console": "warn",
      "max-params": ["warn", 4],
      "complexity": ["warn", 15],
      "no-param-reassign": "error",
      "no-throw-literal": "error",
      "no-eval": "error",
      "no-implied-eval": "error"
    }
  }
];
```

```jsonc
// .prettierrc
{
  "singleQuote": true,
  "trailingComma": "all",
  "semi": true,
  "printWidth": 100,
  "tabWidth": 2
}
```

---

## How to Use

This skill provides rule identifiers for quick reference. When generating or reviewing JavaScript code:

1. **Check relevant category** based on task type
2. **Apply rules** with matching prefix
3. **Prioritize** CRITICAL > HIGH > MEDIUM > LOW
4. **Read rule files** in `rules/` for detailed examples

### Rule Application by Task

| Task | Primary Categories |
|------|-------------------|
| New function/module | `fn-`, `name-`, `doc-` |
| Async code | `async-`, `err-` |
| Error handling | `err-`, `async-` |
| Security review | `sec-`, `node-` |
| API/module design | `mod-`, `type-`, `name-` |
| Performance tuning | `perf-`, `async-`, `node-` |
| Code review | `anti-`, `lint-`, `sec-` |
| Project setup | `proj-`, `lint-`, `node-` |

---

## Related Skills

- [design-patterns](../design-patterns/SKILL.md) — choosing and implementing GoF/idiomatic patterns in JavaScript (module pattern, factory, observer, middleware/pipeline).
- [security-review](../security-review/SKILL.md) — cross-language security/correctness review methodology; its `references/typescript.md` bug-class checklist (prototype pollution, ReDoS, `eval`/injection, dependency confusion) applies directly to plain JavaScript too.
- [typescript-coding](../typescript-coding/SKILL.md) — the sibling skill for TypeScript, useful when the project adds static types on top of this codebase's JavaScript.

## Sources

This skill synthesizes best practices from:
- [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/JavaScript)
- [Node.js Best Practices](https://github.com/goldbergyoni/nodebestpractices)
- [JavaScript Standard Style](https://standardjs.com/)
- [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
- [Node.js Documentation](https://nodejs.org/docs/latest/)
- Production codebases: Express, Fastify, Next.js, Prisma, puppeteer
- ESLint core rules and recommended configs
- TC39 proposals and ES2024 specification
- Community conventions (2024-2025)
