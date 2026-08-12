---
name: typescript-coding
description: "Comprehensive idiomatic TypeScript/JavaScript guidance: 171 prioritized rules across 14 categories. Use when writing, reviewing, refactoring, optimizing, or debugging TypeScript or JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`, `tsconfig.json`, `package.json`). Preserve the target project's declared TypeScript version, module system, and strictness settings; apply `satisfies`, `verbatimModuleSyntax`, `noUncheckedIndexedAccess`, and other modern-config guidance only when the project's toolchain supports them."
compatibility: opencode
metadata:
  domain: typescript
  audience: software-engineer
  edition: project-declared
---

# TypeScript Best Practices

Comprehensive guide for writing high-quality, idiomatic TypeScript and modern JavaScript-via-TypeScript code. Contains 171 rules across 14 categories, prioritized by impact. Project constraints override generic defaults: preserve the declared TypeScript version, `tsconfig.json` strictness settings, module system (ESM/CJS), and target runtime unless the user explicitly requests a migration.

## When to Apply

Reference these guidelines when:
- Writing new TypeScript/JavaScript functions, classes, or modules
- Implementing error handling or async/await code
- Designing public APIs for libraries or shared packages
- Reviewing code for type-safety gaps (`any`, unchecked assertions, missing narrowing)
- Optimizing bundle size, allocations, or hot paths
- Structuring a project, monorepo, or module boundaries
- Writing or reviewing tests (Vitest/Jest)
- Migrating a codebase to stricter `tsconfig.json` settings or newer TypeScript releases

## Modern TypeScript & tsconfig Notes

TypeScript's release cadence (5.4–5.9+ as of 2025) is additive, not edition-based like Rust — there is no single "flip a switch" migration. Preserve a project's existing `tsconfig.json` and only apply the notes below when the installed TypeScript version and runtime actually support them; verify with `npx tsc -v` and the project's `package.json` engines field before assuming a feature is available.

- **`strict: true`.** The single highest-leverage setting. It bundles `strictNullChecks`, `noImplicitAny`, `strictFunctionTypes`, `strictBindCallApply`, `strictPropertyInitialization`, `noImplicitThis`, and `alwaysStrict`. New projects should start here; existing projects should ratchet toward it incrementally (`strictNullChecks` first).
- **`satisfies` operator (5.0+).** Validates that a literal conforms to a type without widening the literal's inferred type the way an `: Type` annotation would — see `type-satisfies-operator`.
- **`verbatimModuleSyntax` (5.0+).** Replaces the older `isolatedModules` + `importsNotUsedAsValues` combination; makes type-only imports/exports explicit (`import type`) so single-file transpilers (esbuild, swc, Babel) never accidentally elide or keep a runtime import incorrectly.
- **`moduleResolution: "bundler"` (5.0+).** Matches how modern bundlers (Vite, esbuild, webpack) actually resolve packages, including `exports` map conditions, without requiring file extensions on relative imports the way `"node16"`/`"nodenext"` does.
- **`noUncheckedIndexedAccess`.** Makes `T[K]` on an index signature or array return `T | undefined` instead of `T`, closing a large hole in `strict` mode around out-of-bounds/missing-key access.
- **Const type parameters (`const T extends ...`, 5.0+).** Infers generic arguments as their literal (`as const`-like) form without callers having to write `as const` themselves at every call site.
- **Template literal types & `as const`.** Together they let you derive precise string-literal unions from data instead of hand-maintaining parallel `enum`/union declarations — see `type-template-literal` and `type-const-assertion`.
- **ESM vs. CJS interop.** Node.js, bundlers, and TypeScript increasingly default to ESM. Use `"type": "module"` in `package.json` for new packages, `import`/`export` syntax, and the `exports` field to declare public entry points; reserve `"type": "commonjs"` and `.cjs` for legacy consumers. Mixed dual-package builds should generate both outputs rather than relying on runtime shims.
- **`isolatedModules`.** Required by every non-`tsc` transpiler (esbuild, swc, Babel) because they compile files independently without full type information; keep it on for any project using such a toolchain.

## Rule Categories by Priority

| Priority | Category | Impact | Prefix | Rules |
|----------|----------|--------|--------|-------|
| 1 | Type Safety & Narrowing | CRITICAL | `type-` | 16 |
| 2 | Error Handling | CRITICAL | `err-` | 13 |
| 3 | Async/Promises/Concurrency | CRITICAL | `async-` | 16 |
| 4 | API/Module Design | HIGH | `api-` | 14 |
| 5 | Immutability & Data Patterns | HIGH | `imm-` | 10 |
| 6 | Functional Patterns | HIGH | `fn-` | 10 |
| 7 | Naming Conventions | MEDIUM | `name-` | 12 |
| 8 | Testing | MEDIUM | `test-` | 13 |
| 9 | Documentation | MEDIUM | `doc-` | 9 |
| 10 | Performance Patterns | MEDIUM | `perf-` | 12 |
| 11 | Node.js/Runtime | MEDIUM | `node-` | 9 |
| 12 | Project Structure & Tooling | LOW | `proj-` | 11 |
| 13 | Linting | LOW | `lint-` | 10 |
| 14 | Anti-patterns | REFERENCE | `anti-` | 16 |

---

## Quick Reference

### 1. Type Safety & Narrowing (CRITICAL)

- [`type-unknown-over-any`](rules/type-unknown-over-any.md) - Use `unknown` instead of `any` for values of uncertain type
- [`type-narrow-guards`](rules/type-narrow-guards.md) - Use user-defined type guards (`is`) to narrow union types safely
- [`type-discriminated-union`](rules/type-discriminated-union.md) - Model variants with discriminated unions and a common tag field
- [`type-exhaustive-switch`](rules/type-exhaustive-switch.md) - Enforce exhaustiveness checks with a `never` assertion
- [`type-satisfies-operator`](rules/type-satisfies-operator.md) - Use `satisfies` to validate a value's shape without widening its type
- [`type-const-assertion`](rules/type-const-assertion.md) - Use `as const` to infer literal, readonly types
- [`type-branded-nominal`](rules/type-branded-nominal.md) - Use branded/nominal types to distinguish primitives with the same runtime type
- [`type-avoid-assertion`](rules/type-avoid-assertion.md) - Avoid `as` type assertions; prefer narrowing or validation
- [`type-strict-null-checks`](rules/type-strict-null-checks.md) - Enable `strictNullChecks` and model absence with `undefined`/`null` explicitly
- [`type-template-literal`](rules/type-template-literal.md) - Use template literal types to constrain string patterns
- [`type-generic-constraints`](rules/type-generic-constraints.md) - Constrain generic type parameters with `extends` instead of leaving them unbounded
- [`type-readonly-arrays`](rules/type-readonly-arrays.md) - Accept `readonly T[]` for parameters that shouldn't be mutated
- [`type-utility-types`](rules/type-utility-types.md) - Prefer built-in utility types (`Pick`, `Omit`, `Partial`, `Required`) over hand-rolled equivalents
- [`type-index-signature-safety`](rules/type-index-signature-safety.md) - Enable `noUncheckedIndexedAccess` and guard indexed access results
- [`type-zod-schema-inference`](rules/type-zod-schema-inference.md) - Derive static types from a runtime schema instead of maintaining both by hand
- [`type-function-overloads`](rules/type-function-overloads.md) - Use overload signatures to model functions with varying call shapes

### 2. Error Handling (CRITICAL)

- [`err-custom-error-class`](rules/err-custom-error-class.md) - Extend `Error` with custom subclasses that carry structured context
- [`err-cause-chaining`](rules/err-cause-chaining.md) - Chain root causes with the standard `cause` option
- [`err-never-swallow`](rules/err-never-swallow.md) - Never silently swallow errors in empty catch blocks
- [`err-result-pattern`](rules/err-result-pattern.md) - Use a `Result`-like return type for expected, recoverable failures
- [`err-async-propagation`](rules/err-async-propagation.md) - Let async/await propagate rejections naturally instead of mixing `.then`/`.catch`
- [`err-promise-allsettled`](rules/err-promise-allsettled.md) - Use `Promise.allSettled` when independent operations may fail without aborting others
- [`err-boundary-validation`](rules/err-boundary-validation.md) - Validate untrusted input at system boundaries with a schema library
- [`err-specific-catch`](rules/err-specific-catch.md) - Catch and handle specific error types instead of a blanket catch-all
- [`err-rethrow-context`](rules/err-rethrow-context.md) - Add context when rethrowing instead of losing the original error
- [`err-no-throw-strings`](rules/err-no-throw-strings.md) - Always throw `Error` instances, never strings or plain objects
- [`err-unhandled-rejection`](rules/err-unhandled-rejection.md) - Register process-level handlers for unhandled promise rejections
- [`err-finally-cleanup`](rules/err-finally-cleanup.md) - Use `finally` for cleanup that must run regardless of outcome
- [`err-typed-catch-unknown`](rules/err-typed-catch-unknown.md) - Type the catch binding as `unknown` and narrow before use

### 3. Async/Promises/Concurrency (CRITICAL)

- [`async-await-over-then`](rules/async-await-over-then.md) - Prefer `async`/`await` over chained `.then()` calls
- [`async-no-floating-promises`](rules/async-no-floating-promises.md) - Never leave a promise floating; await, return, or explicitly void it
- [`async-promise-all-parallel`](rules/async-promise-all-parallel.md) - Use `Promise.all` to run independent async work concurrently
- [`async-avoid-sequential-await`](rules/async-avoid-sequential-await.md) - Avoid awaiting independent operations sequentially inside loops
- [`async-abort-controller`](rules/async-abort-controller.md) - Use `AbortController` to make async operations cancellable
- [`async-timeout-race`](rules/async-timeout-race.md) - Implement timeouts by racing a promise against a timer
- [`async-concurrency-limit`](rules/async-concurrency-limit.md) - Bound concurrency with a limiter when processing large batches
- [`async-no-async-constructor`](rules/async-no-async-constructor.md) - Avoid `async` constructors; use a static async factory method instead
- [`async-for-await-iteration`](rules/async-for-await-iteration.md) - Use `for await...of` to consume async iterables
- [`async-top-level-await`](rules/async-top-level-await.md) - Use top-level `await` only at module entry points
- [`async-avoid-async-foreach`](rules/async-avoid-async-foreach.md) - Avoid `Array.prototype.forEach` with an async callback
- [`async-immediately-invoked`](rules/async-immediately-invoked.md) - Use an async IIFE to run async code in non-async contexts
- [`async-microtask-ordering`](rules/async-microtask-ordering.md) - Understand microtask vs. macrotask ordering to avoid subtle scheduling bugs
- [`async-retry-backoff`](rules/async-retry-backoff.md) - Retry transient failures with exponential backoff and jitter
- [`async-void-operator`](rules/async-void-operator.md) - Use the `void` operator to mark an intentionally ignored promise
- [`async-generator-streams`](rules/async-generator-streams.md) - Use async generators to model lazy, pull-based async sequences

### 4. API/Module Design (HIGH)

- [`api-minimal-surface`](rules/api-minimal-surface.md) - Keep the public API surface as small as the consumer actually needs
- [`api-named-over-default-export`](rules/api-named-over-default-export.md) - Prefer named exports over default exports
- [`api-barrel-file-tradeoffs`](rules/api-barrel-file-tradeoffs.md) - Use barrel (`index.ts`) files judiciously; they can defeat tree-shaking
- [`api-builder-pattern`](rules/api-builder-pattern.md) - Use a builder/fluent API for objects with many optional construction parameters
- [`api-readonly-public-types`](rules/api-readonly-public-types.md) - Mark public interface properties `readonly` unless mutation is part of the contract
- [`api-explicit-return-types`](rules/api-explicit-return-types.md) - Annotate explicit return types on exported functions
- [`api-avoid-optional-overuse`](rules/api-avoid-optional-overuse.md) - Avoid excessive optional properties; model valid states as required unions instead
- [`api-generic-defaults`](rules/api-generic-defaults.md) - Give generic type parameters sensible defaults where one exists
- [`api-function-overload-order`](rules/api-function-overload-order.md) - Order overload signatures from most specific to most general
- [`api-interface-vs-type`](rules/api-interface-vs-type.md) - Use `interface` for extendable object shapes, `type` for unions/aliases/mapped types
- [`api-accept-narrow-return-wide`](rules/api-accept-narrow-return-wide.md) - Accept the most general input types callers already have; return the most specific types
- [`api-avoid-enum-const-object`](rules/api-avoid-enum-const-object.md) - Prefer literal unions or `as const` objects over `enum`
- [`api-module-boundary-types`](rules/api-module-boundary-types.md) - Define explicit DTOs at module/service boundaries, separate from internal domain models
- [`api-versioned-public-api`](rules/api-versioned-public-api.md) - Version public package APIs deliberately and follow semver for breaking changes

### 5. Immutability & Data Patterns (HIGH)

- [`imm-prefer-const`](rules/imm-prefer-const.md) - Default to `const`; use `let` only when a binding is reassigned
- [`imm-as-const-literal`](rules/imm-as-const-literal.md) - Freeze literal object/array structures with `as const`
- [`imm-object-freeze-runtime`](rules/imm-object-freeze-runtime.md) - Use `Object.freeze` when you need a runtime immutability guarantee, not just a compile-time one
- [`imm-spread-not-mutate`](rules/imm-spread-not-mutate.md) - Create updated copies with spread/rest instead of mutating in place
- [`imm-avoid-array-mutation`](rules/imm-avoid-array-mutation.md) - Avoid mutating array methods (`push`, `splice`, `sort`) on shared/shared-reference arrays
- [`imm-structural-sharing`](rules/imm-structural-sharing.md) - Use structural sharing so immutable updates don't copy untouched subtrees
- [`imm-readonly-class-fields`](rules/imm-readonly-class-fields.md) - Mark class fields `readonly` when they are set once in the constructor
- [`imm-deep-immutability-types`](rules/imm-deep-immutability-types.md) - Use a deep-readonly utility type for nested immutable state trees
- [`imm-avoid-param-mutation`](rules/imm-avoid-param-mutation.md) - Never mutate a function's input parameters
- [`imm-immutable-collections`](rules/imm-immutable-collections.md) - Consider a persistent/immutable collection library for hot mutation-heavy state paths

### 6. Functional Patterns (HIGH)

- [`fn-pure-functions`](rules/fn-pure-functions.md) - Prefer pure functions with no hidden side effects
- [`fn-array-methods-over-loops`](rules/fn-array-methods-over-loops.md) - Use `map`/`filter`/`reduce` for transformations instead of manual `for` loops
- [`fn-composition-over-inheritance`](rules/fn-composition-over-inheritance.md) - Compose small functions instead of building class inheritance hierarchies
- [`fn-curry-partial-application`](rules/fn-curry-partial-application.md) - Use currying/partial application to produce reusable configured functions
- [`fn-avoid-reduce-abuse`](rules/fn-avoid-reduce-abuse.md) - Avoid `reduce` when a more specific method already expresses the intent
- [`fn-optional-chaining`](rules/fn-optional-chaining.md) - Use optional chaining (`?.`) instead of manual nested null checks
- [`fn-nullish-coalescing`](rules/fn-nullish-coalescing.md) - Use `??` instead of `||` when only `null`/`undefined` should trigger the default
- [`fn-pipeline-composition`](rules/fn-pipeline-composition.md) - Compose sequential data transformations as an explicit pipeline
- [`fn-early-return`](rules/fn-early-return.md) - Use early returns/guard clauses to reduce nesting
- [`fn-avoid-side-effects-in-map`](rules/fn-avoid-side-effects-in-map.md) - Never use `.map()` purely for side effects; use `.forEach()` or a `for` loop

### 7. Naming Conventions (MEDIUM)

- [`name-camelCase-vars`](rules/name-camelCase-vars.md) - Use `camelCase` for variables and functions
- [`name-PascalCase-types`](rules/name-PascalCase-types.md) - Use `PascalCase` for types, interfaces, classes, and enums
- [`name-SCREAMING-const`](rules/name-SCREAMING-const.md) - Use `SCREAMING_SNAKE_CASE` for true module-level constants
- [`name-boolean-prefix`](rules/name-boolean-prefix.md) - Prefix booleans with `is`/`has`/`can`/`should`
- [`name-no-hungarian`](rules/name-no-hungarian.md) - Avoid Hungarian notation and redundant type suffixes in identifier names
- [`name-verb-noun-functions`](rules/name-verb-noun-functions.md) - Name functions with a leading verb describing the action they perform
- [`name-avoid-abbreviations`](rules/name-avoid-abbreviations.md) - Avoid unclear abbreviations in identifiers
- [`name-private-underscore-avoid`](rules/name-private-underscore-avoid.md) - Use `private`/`#` for privacy instead of a leading-underscore convention
- [`name-generic-type-params`](rules/name-generic-type-params.md) - Use conventional short generic names (`T`, `K`, `V`, `E`) or a descriptive name for complex generics
- [`name-file-naming-convention`](rules/name-file-naming-convention.md) - Apply one consistent file naming convention (kebab-case or PascalCase) per project
- [`name-interface-no-I-prefix`](rules/name-interface-no-I-prefix.md) - Don't prefix interfaces with `I`
- [`name-async-suffix-when-ambiguous`](rules/name-async-suffix-when-ambiguous.md) - Suffix an async function's name when a sync counterpart exists with the same base name

### 8. Testing (MEDIUM)

- [`test-arrange-act-assert`](rules/test-arrange-act-assert.md) - Structure tests as arrange/act/assert
- [`test-descriptive-names`](rules/test-descriptive-names.md) - Name tests descriptively: "should X when Y"
- [`test-vitest-jest-setup`](rules/test-vitest-jest-setup.md) - Follow standard Vitest/Jest project conventions for config and structure
- [`test-mock-boundaries`](rules/test-mock-boundaries.md) - Mock external boundaries (network, filesystem, clock), not internal implementation details
- [`test-avoid-snapshot-abuse`](rules/test-avoid-snapshot-abuse.md) - Use snapshot tests sparingly, and review generated snapshots deliberately
- [`test-async-test-patterns`](rules/test-async-test-patterns.md) - Always await async assertions; never leave a test's promise unhandled
- [`test-test-doubles`](rules/test-test-doubles.md) - Choose the right test double: stub, spy, mock, or fake
- [`test-isolate-tests`](rules/test-isolate-tests.md) - Keep tests isolated and order-independent, with no shared mutable state
- [`test-coverage-meaningful`](rules/test-coverage-meaningful.md) - Target meaningful coverage of behavior, not a 100% coverage vanity metric
- [`test-integration-vs-unit`](rules/test-integration-vs-unit.md) - Balance the test pyramid between unit and integration tests
- [`test-fixture-factories`](rules/test-fixture-factories.md) - Use factory functions to build test fixtures instead of duplicating literals
- [`test-parameterized-tests`](rules/test-parameterized-tests.md) - Use parameterized/table-driven tests (`it.each`) for input/output variants
- [`test-fake-timers`](rules/test-fake-timers.md) - Use fake timers to test time-dependent code deterministically

### 9. Documentation (MEDIUM)

- [`doc-tsdoc-public-api`](rules/doc-tsdoc-public-api.md) - Document all public API with TSDoc comments
- [`doc-example-tags`](rules/doc-example-tags.md) - Include an `@example` block in non-trivial doc comments
- [`doc-param-returns-tags`](rules/doc-param-returns-tags.md) - Document `@param`/`@returns` for signatures that aren't self-evident
- [`doc-deprecated-tag`](rules/doc-deprecated-tag.md) - Mark deprecated APIs with `@deprecated` and a migration path
- [`doc-readme-package`](rules/doc-readme-package.md) - Maintain a README with install/usage examples for every published package
- [`doc-changelog-semver`](rules/doc-changelog-semver.md) - Maintain a CHANGELOG that follows semantic versioning
- [`doc-inline-why-not-what`](rules/doc-inline-why-not-what.md) - Write comments that explain why, not what the code already says
- [`doc-type-as-documentation`](rules/doc-type-as-documentation.md) - Let precise types replace comments that only describe a shape
- [`doc-throws-tag`](rules/doc-throws-tag.md) - Document the errors a function can throw with `@throws`

### 10. Performance Patterns (MEDIUM)

- [`perf-avoid-premature-optimize`](rules/perf-avoid-premature-optimize.md) - Profile before optimizing
- [`perf-tree-shaking-friendly`](rules/perf-tree-shaking-friendly.md) - Write side-effect-free modules so bundlers can tree-shake unused exports
- [`perf-lazy-load-dynamic-import`](rules/perf-lazy-load-dynamic-import.md) - Use dynamic `import()` for code splitting and lazy loading
- [`perf-avoid-unnecessary-allocation`](rules/perf-avoid-unnecessary-allocation.md) - Avoid allocating objects/arrays inside hot loops
- [`perf-memoize-expensive`](rules/perf-memoize-expensive.md) - Memoize expensive pure computations
- [`perf-debounce-throttle`](rules/perf-debounce-throttle.md) - Debounce or throttle high-frequency event handlers
- [`perf-avoid-json-parse-large`](rules/perf-avoid-json-parse-large.md) - Avoid blocking synchronous JSON parsing of large payloads on hot paths
- [`perf-string-concat-builder`](rules/perf-string-concat-builder.md) - Build large strings with arrays/template literals, not repeated `+=` concatenation
- [`perf-avoid-deep-clone`](rules/perf-avoid-deep-clone.md) - Avoid deep cloning when structural sharing or shallow copies suffice
- [`perf-bundle-size-audit`](rules/perf-bundle-size-audit.md) - Audit bundle size and dependency weight regularly
- [`perf-worker-offload`](rules/perf-worker-offload.md) - Offload CPU-heavy work to a worker thread instead of blocking the main thread
- [`perf-avoid-blocking-event-loop`](rules/perf-avoid-blocking-event-loop.md) - Avoid long synchronous operations that block the event loop

### 11. Node.js/Runtime (MEDIUM)

- [`node-esm-first`](rules/node-esm-first.md) - Prefer ES modules over CommonJS for new Node.js projects
- [`node-package-exports-map`](rules/node-package-exports-map.md) - Define package entry points with the `exports` field
- [`node-env-var-validation`](rules/node-env-var-validation.md) - Validate environment variables against a schema at startup
- [`node-graceful-shutdown`](rules/node-graceful-shutdown.md) - Handle `SIGTERM`/`SIGINT` for graceful shutdown
- [`node-streams-backpressure`](rules/node-streams-backpressure.md) - Use streams with backpressure for large I/O instead of buffering in memory
- [`node-avoid-sync-fs`](rules/node-avoid-sync-fs.md) - Avoid synchronous `fs` calls on a server's request path
- [`node-process-exit-avoid`](rules/node-process-exit-avoid.md) - Avoid `process.exit()` in library code; let the caller control the process
- [`node-worker-threads-cpu`](rules/node-worker-threads-cpu.md) - Use `worker_threads` for CPU-bound work in a Node.js server
- [`node-structured-logging`](rules/node-structured-logging.md) - Use structured, leveled logging instead of `console.log`

### 12. Project Structure & Tooling (LOW)

- [`proj-path-aliases`](rules/proj-path-aliases.md) - Use `tsconfig` path aliases instead of long relative import chains
- [`proj-monorepo-workspaces`](rules/proj-monorepo-workspaces.md) - Use workspaces (pnpm/npm/yarn) to manage a monorepo's packages
- [`proj-feature-based-structure`](rules/proj-feature-based-structure.md) - Organize source by feature/domain, not by technical file type
- [`proj-single-tsconfig-base`](rules/proj-single-tsconfig-base.md) - Share a base `tsconfig.json` and extend it per package
- [`proj-module-boundaries`](rules/proj-module-boundaries.md) - Enforce module boundaries; don't import another module's internal files
- [`proj-colocate-tests`](rules/proj-colocate-tests.md) - Colocate tests with source, or mirror source structure consistently — pick one
- [`proj-env-specific-config`](rules/proj-env-specific-config.md) - Keep environment-specific configuration separate from code
- [`proj-verbatim-module-syntax`](rules/proj-verbatim-module-syntax.md) - Enable `verbatimModuleSyntax` for unambiguous type-only imports/exports
- [`proj-isolated-modules`](rules/proj-isolated-modules.md) - Enable `isolatedModules` for compatibility with single-file transpilers
- [`proj-declaration-files`](rules/proj-declaration-files.md) - Emit `.d.ts` declaration files for any published package
- [`proj-lockfile-commit`](rules/proj-lockfile-commit.md) - Commit the lockfile for reproducible installs

### 13. Linting (LOW)

- [`lint-typescript-eslint-recommended`](rules/lint-typescript-eslint-recommended.md) - Adopt `typescript-eslint`'s recommended (or recommended-type-checked) config
- [`lint-no-explicit-any`](rules/lint-no-explicit-any.md) - Enable `@typescript-eslint/no-explicit-any`
- [`lint-no-floating-promises-rule`](rules/lint-no-floating-promises-rule.md) - Enable `@typescript-eslint/no-floating-promises`
- [`lint-no-non-null-assertion`](rules/lint-no-non-null-assertion.md) - Enable `@typescript-eslint/no-non-null-assertion`
- [`lint-prettier-integration`](rules/lint-prettier-integration.md) - Use Prettier for formatting and let ESLint own only code-quality rules
- [`lint-strict-tsconfig`](rules/lint-strict-tsconfig.md) - Enable `strict: true` and other strictness flags in `tsconfig.json`
- [`lint-no-unused-vars`](rules/lint-no-unused-vars.md) - Enable the TypeScript-aware `no-unused-vars` rule
- [`lint-consistent-type-imports`](rules/lint-consistent-type-imports.md) - Enforce `consistent-type-imports` so type-only imports are marked explicitly
- [`lint-ci-lint-gate`](rules/lint-ci-lint-gate.md) - Run typecheck and lint as a required CI gate
- [`lint-no-unchecked-indexed-access`](rules/lint-no-unchecked-indexed-access.md) - Enable `noUncheckedIndexedAccess` in `tsconfig.json`

### 14. Anti-patterns (REFERENCE)

- [`anti-any-abuse`](rules/anti-any-abuse.md) - Don't use `any` to silence type errors
- [`anti-non-null-assertion-abuse`](rules/anti-non-null-assertion-abuse.md) - Don't overuse the `!` non-null assertion operator
- [`anti-loose-equality`](rules/anti-loose-equality.md) - Don't use `==`/`!=`; use `===`/`!==`
- [`anti-callback-hell`](rules/anti-callback-hell.md) - Don't nest callbacks; use async/await instead
- [`anti-mutate-props-state`](rules/anti-mutate-props-state.md) - Don't mutate props or shared state objects directly
- [`anti-for-in-arrays`](rules/anti-for-in-arrays.md) - Don't use `for...in` to iterate arrays
- [`anti-stringly-typed-data`](rules/anti-stringly-typed-data.md) - Don't represent structured data as ad hoc strings
- [`anti-god-object`](rules/anti-god-object.md) - Don't build "God" objects/functions with too many responsibilities
- [`anti-magic-numbers`](rules/anti-magic-numbers.md) - Don't scatter unexplained magic numbers/strings through code
- [`anti-var-usage`](rules/anti-var-usage.md) - Don't use `var`; use `const`/`let`
- [`anti-empty-catch-block`](rules/anti-empty-catch-block.md) - Don't leave catch blocks empty
- [`anti-any-cast-double`](rules/anti-any-cast-double.md) - Don't force incorrect types with `as unknown as T` double casts
- [`anti-deeply-nested-ternary`](rules/anti-deeply-nested-ternary.md) - Don't nest ternary expressions deeply
- [`anti-global-mutable-state`](rules/anti-global-mutable-state.md) - Don't rely on global mutable state
- [`anti-promise-constructor-antipattern`](rules/anti-promise-constructor-antipattern.md) - Don't wrap an already-promise-returning call in `new Promise`
- [`anti-type-any-return`](rules/anti-type-any-return.md) - Don't return `any` from a function; it erases type safety for every caller

---

## Recommended tsconfig.json / package.json Settings

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",       // or "nodenext" for a Node.js-only library
    "lib": ["ES2022"],
    "strict": true,                       // strictNullChecks, noImplicitAny, etc.
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "verbatimModuleSyntax": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "skipLibCheck": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "dist",
    "rootDir": "src"
  },
  "include": ["src"],
  "exclude": ["dist", "node_modules"]
}
```

```json
// package.json (relevant fields for an ESM-first published package)
{
  "type": "module",
  "engines": { "node": ">=20" },
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  },
  "files": ["dist"],
  "sideEffects": false,
  "scripts": {
    "build": "tsc -p tsconfig.build.json",
    "typecheck": "tsc --noEmit",
    "lint": "eslint . --max-warnings 0",
    "test": "vitest run"
  }
}
```

---

## How to Use

This skill provides rule identifiers for quick reference. When generating or reviewing TypeScript/JavaScript code:

1. **Check relevant category** based on task type
2. **Apply rules** with matching prefix
3. **Prioritize** CRITICAL > HIGH > MEDIUM > LOW
4. **Read rule files** in `rules/` for detailed examples

### Rule Application by Task

| Task | Primary Categories |
|------|-------------------|
| New function/module | `type-`, `err-`, `name-` |
| New public API/package | `api-`, `type-`, `doc-` |
| Async/network code | `async-`, `err-` |
| Error handling | `err-`, `type-` |
| State management | `imm-`, `fn-` |
| Performance tuning | `perf-`, `async-`, `node-` |
| Writing tests | `test-` |
| Project/monorepo setup | `proj-`, `lint-`, `node-` |
| Code review | `anti-`, `lint-` |

---

## Related Skills

- [design-patterns](../design-patterns/SKILL.md) - choosing and implementing GoF and idiomatic patterns; apply alongside this skill's API and naming rules for pattern-heavy TypeScript design.
- [security-review](../security-review/SKILL.md) - security-focused audit checklists; apply alongside this skill's error-handling and async rules when reviewing TypeScript code for vulnerabilities.

## Sources

This skill synthesizes best practices from:
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html) and the official [TSConfig Reference](https://www.typescriptlang.org/tsconfig)
- [typescript-eslint](https://typescript-eslint.io/rules/) rule documentation
- [Google TypeScript Style Guide](https://google.github.io/styleguide/tsguide.html)
- [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
- *Effective TypeScript* (Dan Vanderkam) and *Total TypeScript* (Matt Pocock)
- Production codebases: `type-fest`, `zod`, `trpc`, `vue`, `react`, `vite`
- Node.js and TC39/ECMAScript proposal documentation
- Community conventions (2024-2025)
