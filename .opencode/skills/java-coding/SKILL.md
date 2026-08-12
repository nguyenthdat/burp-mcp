---
name: java-coding
description: "Comprehensive idiomatic Java guidance: 172 prioritized rules across 14 categories covering null safety/Optional, exceptions, virtual threads and concurrency, modern language features (records, sealed types, pattern matching), and API design. Use when writing, reviewing, refactoring, optimizing, or debugging Java (`.java`, `pom.xml`, `build.gradle`/`build.gradle.kts`) code. Preserve the target project's declared Java version (17/21/25 LTS) and build toolchain; apply virtual-thread, structured-concurrency, and pattern-matching guidance only when the project's language level actually supports them."
compatibility: opencode
metadata:
  domain: java
  audience: software-engineer
  edition: project-declared
---

# Java Best Practices

Comprehensive guide for writing high-quality, idiomatic modern Java code across Java SE, concurrency, and library/application design. Contains 172 rules across 14 categories, prioritized by impact. Project constraints override generic defaults: preserve the declared Java language level (`<release>`/`sourceCompatibility`/`toolchain`), build tool (Maven/Gradle), and supported runtime targets unless the user explicitly requests a migration.

## When to Apply

Reference these guidelines when:
- Writing new Java classes, records, interfaces, or modules
- Implementing error handling, checked/unchecked exception hierarchies, or resource cleanup
- Writing concurrent code with virtual threads, `ExecutorService`, or `CompletableFuture`
- Designing public APIs for libraries or multi-module Maven/Gradle projects
- Reviewing code for null-safety gaps (missing `Optional`, unguarded nulls, nullable leakage)
- Migrating switch statements, data classes, or type hierarchies to modern pattern-matching idioms
- Optimizing allocations, boxing, or reflection overhead on hot paths
- Structuring a Maven/Gradle multi-module project or JPMS module boundary
- Writing or reviewing tests (JUnit 5, Mockito, AssertJ)
- Setting up static analysis (Checkstyle, SpotBugs, Error Prone, PMD) in CI

## Java 17/21/25 LTS Notes

Java 17, 21, and 25 are the current LTS releases (25 released September 2025). Preserve a project's existing `--release`/toolchain version and only apply the notes below when the declared language level actually supports them; verify with the project's `pom.xml`/`build.gradle(.kts)` before assuming a feature is available.

- **Records (16+, stable).** Compact, immutable data carriers with generated `equals`/`hashCode`/`toString`/accessors — see `api-record-data-carrier`, `modern-records-immutable-data`. Validate invariants in a compact constructor (`modern-records-immutable-data`, `api-record-compact-constructor-validation`).
- **Sealed classes/interfaces (17+, stable).** Restrict which types may implement/extend a hierarchy with `sealed`/`permits`, pairing naturally with exhaustive `switch` — see `api-sealed-closed-hierarchy`, `modern-sealed-interfaces-hierarchy`.
- **Pattern matching for `switch` and record patterns (21+, stable).** `switch` can match on type patterns, record deconstruction patterns, and `null`, with `when` guard clauses for refinement — see `type-pattern-matching-switch`, `modern-record-deconstruction-patterns`, `modern-guarded-patterns-when`.
- **Virtual threads (21+, stable, JEP 444).** Lightweight, JVM-managed threads let blocking, thread-per-request-style I/O code scale to millions of concurrent tasks without reactive rewrites — see `conc-virtual-threads-io`, `modern-virtual-threads-jep444`. Watch for **pinning**: a virtual thread pinned inside a `synchronized` block (or a native frame) blocks its carrier platform thread — see `conc-avoid-pinning`, `anti-blocking-call-on-virtual-thread-pinning`.
- **Structured concurrency (25: fourth preview, still `--enable-preview`).** `StructuredTaskScope` treats a group of related subtasks launched in different threads as a single unit of work with unified cancellation, shutdown, and error propagation — see `conc-structured-concurrency`, `modern-structured-concurrency-preview`. Confirm the project has opted into preview features before relying on the final API shape.
- **Scoped values (25: fifth preview).** `ScopedValue` shares immutable data within and across threads (including virtual threads spawned by structured concurrency) without the mutation and unbounded-lifetime hazards of `ThreadLocal` — see `conc-scoped-values-not-threadlocal`, `modern-scoped-values`.
- **Sequenced collections (21+, stable, JEP 431).** `SequencedCollection`, `SequencedSet`, and `SequencedMap` add well-defined encounter-order operations (`getFirst`, `getLast`, `reversed`) across existing collection types — see `modern-sequenced-collections`.
- **Text blocks (15+, stable).** Triple-quoted multi-line string literals with predictable indentation stripping, ideal for embedded SQL/JSON/HTML — see `modern-text-blocks`.
- **`var` local-variable type inference (10+, stable).** Use for local variables only, when the initializer already makes the type obvious; never for fields, parameters, or return types — see `type-var-inference-readability`, `modern-var-local-inference`.
- **Foreign Function & Memory API (22+, stable, JEP 454).** Replaces JNI for calling native code and managing off-heap memory safely — see `modern-foreign-function-memory-api`.

For the authoritative, complete list, consult the OpenJDK JEP index and the release notes for the project's target LTS. Everything below applies across Java 17+; prefer the 21/25 forms above where the project's toolchain supports them.

## Rule Categories by Priority

| Priority | Category | Impact | Prefix | Rules |
|----------|----------|--------|--------|-------|
| 1 | Null Safety & Optionals | CRITICAL | `null-` | 12 |
| 2 | Error Handling & Exceptions | CRITICAL | `err-` | 13 |
| 3 | Concurrency & Virtual Threads | CRITICAL | `conc-` | 15 |
| 4 | Object/API Design | HIGH | `api-` | 15 |
| 5 | Generics & Type Safety | HIGH | `type-` | 12 |
| 6 | Modern Java Features | HIGH | `modern-` | 12 |
| 7 | Collections & Streams | HIGH | `coll-` | 14 |
| 8 | Naming Conventions | MEDIUM | `name-` | 10 |
| 9 | Testing | MEDIUM | `test-` | 13 |
| 10 | Documentation | MEDIUM | `doc-` | 9 |
| 11 | Performance Patterns | MEDIUM | `perf-` | 12 |
| 12 | Project Structure | LOW | `proj-` | 10 |
| 13 | Linting & Static Analysis | LOW | `lint-` | 10 |
| 14 | Anti-patterns | REFERENCE | `anti-` | 15 |

---

## Quick Reference

### 1. Null Safety & Optionals (CRITICAL)

- [`null-optional-return-type`](rules/null-optional-return-type.md) - Use `Optional<T>` for return types, never for fields or parameters
- [`null-no-optional-field`](rules/null-no-optional-field.md) - Never declare a field of type `Optional<T>`
- [`null-no-optional-param`](rules/null-no-optional-param.md) - Never accept `Optional<T>` as a method parameter
- [`null-requireNonNull-guard`](rules/null-requireNonNull-guard.md) - Guard constructors/methods with `Objects.requireNonNull`
- [`null-empty-collection-not-null`](rules/null-empty-collection-not-null.md) - Return empty collections, never `null`, from collection-returning methods
- [`null-nullable-annotation`](rules/null-nullable-annotation.md) - Annotate nullability with `@Nullable`/`@NonNull` (JSpecify)
- [`null-optional-chaining`](rules/null-optional-chaining.md) - Chain `Optional` with `map`/`flatMap`/`filter` instead of manual null checks
- [`null-optional-orElseThrow`](rules/null-optional-orElseThrow.md) - Use `orElseThrow` with a meaningful exception supplier
- [`null-avoid-isPresent-get`](rules/null-avoid-isPresent-get.md) - Avoid `isPresent()` + `get()`; use functional `Optional` methods
- [`null-optional-primitive`](rules/null-optional-primitive.md) - Use `OptionalInt`/`OptionalLong`/`OptionalDouble` for primitives
- [`null-defensive-copy`](rules/null-defensive-copy.md) - Defensive-copy mutable fields to prevent aliasing/null surprises
- [`null-jspecify-nullmarked`](rules/null-jspecify-nullmarked.md) - Adopt `@NullMarked` (JSpecify) at the package level

### 2. Error Handling & Exceptions (CRITICAL)

- [`err-checked-vs-unchecked`](rules/err-checked-vs-unchecked.md) - Choose checked vs unchecked exceptions by recoverability
- [`err-custom-exception-hierarchy`](rules/err-custom-exception-hierarchy.md) - Build a custom exception hierarchy for domain errors
- [`err-try-with-resources`](rules/err-try-with-resources.md) - Use try-with-resources for every `AutoCloseable`
- [`err-no-catch-broad`](rules/err-no-catch-broad.md) - Don't catch `Exception`/`Throwable` broadly
- [`err-exception-chaining`](rules/err-exception-chaining.md) - Chain causes via constructor, not `initCause` after the fact
- [`err-no-control-flow`](rules/err-no-control-flow.md) - Don't use exceptions for ordinary control flow
- [`err-fail-fast-validation`](rules/err-fail-fast-validation.md) - Validate arguments early and fail fast
- [`err-specific-catch-order`](rules/err-specific-catch-order.md) - Catch specific exceptions before general ones; use multi-catch
- [`err-finally-vs-twr`](rules/err-finally-vs-twr.md) - Prefer try-with-resources over manual `finally` cleanup
- [`err-exception-message-context`](rules/err-exception-message-context.md) - Include actionable context in exception messages
- [`err-no-empty-catch`](rules/err-no-empty-catch.md) - Never swallow exceptions silently
- [`err-unchecked-wrap-checked`](rules/err-unchecked-wrap-checked.md) - Wrap checked exceptions instead of propagating `throws` everywhere
- [`err-suppressed-exceptions`](rules/err-suppressed-exceptions.md) - Preserve suppressed exceptions from try-with-resources

### 3. Concurrency & Virtual Threads (CRITICAL)

- [`conc-virtual-threads-io`](rules/conc-virtual-threads-io.md) - Use virtual threads for I/O-bound concurrent tasks
- [`conc-platform-threads-cpu`](rules/conc-platform-threads-cpu.md) - Reserve platform threads for CPU-bound work
- [`conc-executors-newVirtualThreadPerTask`](rules/conc-executors-newVirtualThreadPerTask.md) - Use `Executors.newVirtualThreadPerTaskExecutor()`
- [`conc-structured-concurrency`](rules/conc-structured-concurrency.md) - Use `StructuredTaskScope` to manage related task groups
- [`conc-avoid-shared-mutable-state`](rules/conc-avoid-shared-mutable-state.md) - Avoid shared mutable state; prefer immutability/confinement
- [`conc-concurrent-collections`](rules/conc-concurrent-collections.md) - Use `java.util.concurrent` collections over manual synchronization
- [`conc-synchronized-scope`](rules/conc-synchronized-scope.md) - Keep `synchronized` blocks minimal and scoped
- [`conc-completablefuture-composition`](rules/conc-completablefuture-composition.md) - Compose async work with `CompletableFuture`
- [`conc-avoid-pinning`](rules/conc-avoid-pinning.md) - Avoid virtual-thread pinning (`synchronized`, native frames)
- [`conc-scoped-values-not-threadlocal`](rules/conc-scoped-values-not-threadlocal.md) - Prefer `ScopedValue` over unscoped `ThreadLocal` with virtual threads
- [`conc-atomic-over-lock`](rules/conc-atomic-over-lock.md) - Use atomic classes for simple counters instead of locks
- [`conc-executorservice-shutdown`](rules/conc-executorservice-shutdown.md) - Always shut down an `ExecutorService` properly
- [`conc-countdownlatch-cyclicbarrier`](rules/conc-countdownlatch-cyclicbarrier.md) - Use `CountDownLatch`/`CyclicBarrier` for coordination
- [`conc-immutable-thread-safety`](rules/conc-immutable-thread-safety.md) - Make shared objects immutable for thread safety
- [`conc-holder-idiom-lazy-singleton`](rules/conc-holder-idiom-lazy-singleton.md) - Use the initialization-on-demand holder idiom for lazy singletons

### 4. Object/API Design (HIGH)

- [`api-record-data-carrier`](rules/api-record-data-carrier.md) - Use records for immutable data carriers
- [`api-builder-complex-construction`](rules/api-builder-complex-construction.md) - Use the Builder pattern for complex construction
- [`api-sealed-closed-hierarchy`](rules/api-sealed-closed-hierarchy.md) - Use sealed classes/interfaces for closed hierarchies
- [`api-composition-over-inheritance`](rules/api-composition-over-inheritance.md) - Favor composition over inheritance
- [`api-minimal-public-surface`](rules/api-minimal-public-surface.md) - Default to package-private; keep the public surface minimal
- [`api-immutable-by-default`](rules/api-immutable-by-default.md) - Design objects immutable by default
- [`api-equals-hashcode-contract`](rules/api-equals-hashcode-contract.md) - Honor the `equals`/`hashCode` contract together
- [`api-tostring-diagnostics`](rules/api-tostring-diagnostics.md) - Override `toString()` for diagnostics
- [`api-interface-default-methods`](rules/api-interface-default-methods.md) - Prefer interfaces with default methods over rigid abstract base classes
- [`api-final-classes-not-designed-for-inheritance`](rules/api-final-classes-not-designed-for-inheritance.md) - Mark classes `final` unless designed and documented for inheritance
- [`api-static-factory-over-constructor`](rules/api-static-factory-over-constructor.md) - Use static factory methods for readability and caching
- [`api-fluent-method-chaining`](rules/api-fluent-method-chaining.md) - Design fluent, chainable APIs deliberately
- [`api-defensive-copy-mutable-args`](rules/api-defensive-copy-mutable-args.md) - Defensive-copy mutable constructor arguments
- [`api-avoid-telescoping-constructors`](rules/api-avoid-telescoping-constructors.md) - Avoid telescoping constructors; use a builder
- [`api-record-compact-constructor-validation`](rules/api-record-compact-constructor-validation.md) - Validate invariants in a record's compact constructor

### 5. Generics & Type Safety (HIGH)

- [`type-bounded-wildcards-pecs`](rules/type-bounded-wildcards-pecs.md) - Use bounded wildcards (PECS: producer-extends, consumer-super)
- [`type-avoid-raw-types`](rules/type-avoid-raw-types.md) - Avoid raw types; always parameterize generics
- [`type-generic-method-inference`](rules/type-generic-method-inference.md) - Rely on generic method type inference over explicit witnesses
- [`type-avoid-unchecked-cast`](rules/type-avoid-unchecked-cast.md) - Isolate unchecked casts behind a documented `@SuppressWarnings`
- [`type-pattern-matching-instanceof`](rules/type-pattern-matching-instanceof.md) - Use pattern matching for `instanceof`
- [`type-pattern-matching-switch`](rules/type-pattern-matching-switch.md) - Use pattern matching for `switch` over sealed types
- [`type-generic-array-avoid`](rules/type-generic-array-avoid.md) - Avoid generic array creation; use a `List` instead
- [`type-bridge-method-awareness`](rules/type-bridge-method-awareness.md) - Understand bridge methods and erasure implications
- [`type-recursive-generic-bound`](rules/type-recursive-generic-bound.md) - Use recursive generic bounds for self-referencing builders
- [`type-var-inference-readability`](rules/type-var-inference-readability.md) - Use `var` only when the type is obvious from context
- [`type-enum-over-int-constants`](rules/type-enum-over-int-constants.md) - Use enums instead of int constants
- [`type-safevarargs-heap-pollution`](rules/type-safevarargs-heap-pollution.md) - Guard varargs generics with a correct `@SafeVarargs`

### 6. Modern Java Features (HIGH)

- [`modern-records-immutable-data`](rules/modern-records-immutable-data.md) - Use records instead of hand-written immutable data classes
- [`modern-sealed-interfaces-hierarchy`](rules/modern-sealed-interfaces-hierarchy.md) - Use sealed interfaces to model closed hierarchies
- [`modern-switch-expressions`](rules/modern-switch-expressions.md) - Use arrow-form switch expressions over statement switches
- [`modern-record-deconstruction-patterns`](rules/modern-record-deconstruction-patterns.md) - Use record patterns to deconstruct records in `switch`/`instanceof`
- [`modern-text-blocks`](rules/modern-text-blocks.md) - Use text blocks for multi-line strings
- [`modern-var-local-inference`](rules/modern-var-local-inference.md) - Use `var` for local variable type inference where it aids readability
- [`modern-virtual-threads-jep444`](rules/modern-virtual-threads-jep444.md) - Adopt virtual threads (JEP 444) for scalable concurrency
- [`modern-sequenced-collections`](rules/modern-sequenced-collections.md) - Use `SequencedCollection`/`SequencedMap`/`SequencedSet` APIs
- [`modern-structured-concurrency-preview`](rules/modern-structured-concurrency-preview.md) - Use structured concurrency (preview) for related task groups
- [`modern-scoped-values`](rules/modern-scoped-values.md) - Use `ScopedValue` instead of `ThreadLocal` for immutable context propagation
- [`modern-guarded-patterns-when`](rules/modern-guarded-patterns-when.md) - Use guarded patterns (`when` clauses) in `switch`
- [`modern-foreign-function-memory-api`](rules/modern-foreign-function-memory-api.md) - Use the Foreign Function & Memory API instead of JNI

### 7. Collections & Streams (HIGH)

- [`coll-choose-right-collection`](rules/coll-choose-right-collection.md) - Choose the right collection type for the access pattern
- [`coll-immutable-factories`](rules/coll-immutable-factories.md) - Use `List.of`/`Map.of`/`Set.of` immutable factories
- [`coll-avoid-legacy-classes`](rules/coll-avoid-legacy-classes.md) - Avoid legacy `Vector`/`Hashtable`/`Stack` classes
- [`coll-stream-for-transformation`](rules/coll-stream-for-transformation.md) - Use the Stream API for transformation pipelines
- [`coll-avoid-side-effects-streams`](rules/coll-avoid-side-effects-streams.md) - Avoid side effects inside stream operations
- [`coll-collectors-toX`](rules/coll-collectors-toX.md) - Use the right `Collectors` (`toList`, `toMap`, `groupingBy`, `joining`)
- [`coll-stream-vs-loop`](rules/coll-stream-vs-loop.md) - Choose streams vs loops based on readability and performance
- [`coll-primitive-streams-hot-path`](rules/coll-primitive-streams-hot-path.md) - Use primitive streams (`IntStream`, etc.) on hot paths
- [`coll-unmodifiable-view`](rules/coll-unmodifiable-view.md) - Wrap mutable collections with unmodifiable views at boundaries
- [`coll-comparator-composition`](rules/coll-comparator-composition.md) - Compose `Comparator`s with `comparing`/`thenComparing`
- [`coll-removeIf-over-iterator`](rules/coll-removeIf-over-iterator.md) - Use `removeIf` instead of manual iterator removal
- [`coll-map-computeIfAbsent-merge`](rules/coll-map-computeIfAbsent-merge.md) - Use `Map.computeIfAbsent`/`merge` for insert-or-update
- [`coll-stream-parallel-caution`](rules/coll-stream-parallel-caution.md) - Use parallel streams only after profiling, on splittable CPU-bound work
- [`coll-collection-factory-vs-loop`](rules/coll-collection-factory-vs-loop.md) - Prefer `of()`/`copyOf()` factories over manual population loops

### 8. Naming Conventions (MEDIUM)

- [`name-classes-pascal`](rules/name-classes-pascal.md) - Use `PascalCase` for classes, interfaces, enums, records
- [`name-methods-camel`](rules/name-methods-camel.md) - Use `camelCase` for methods and fields
- [`name-constants-screaming-snake`](rules/name-constants-screaming-snake.md) - Use `SCREAMING_SNAKE_CASE` for `static final` constants
- [`name-packages-lowercase`](rules/name-packages-lowercase.md) - Use all-lowercase, reverse-domain package names
- [`name-type-param-single-letter`](rules/name-type-param-single-letter.md) - Use single uppercase letters for generic type parameters
- [`name-boolean-is-has-can`](rules/name-boolean-is-has-can.md) - Name boolean accessors `isX`/`hasX`/`canX`
- [`name-acronyms-as-words`](rules/name-acronyms-as-words.md) - Treat acronyms as words: `HttpClient`, not `HTTPClient`
- [`name-getter-setter-bean-convention`](rules/name-getter-setter-bean-convention.md) - Follow JavaBean getter/setter naming for accessors
- [`name-no-hungarian-notation`](rules/name-no-hungarian-notation.md) - Avoid Hungarian notation and type-suffix cruft
- [`name-test-method-descriptive`](rules/name-test-method-descriptive.md) - Name test methods descriptively, not `test1`

### 9. Testing (MEDIUM)

- [`test-junit5-annotations`](rules/test-junit5-annotations.md) - Use JUnit 5 (`@Test`, `@Nested`, `@DisplayName`) idiomatically
- [`test-parameterized-tests`](rules/test-parameterized-tests.md) - Use `@ParameterizedTest` for input/output variants
- [`test-mockito-mocking`](rules/test-mockito-mocking.md) - Use Mockito for mocking dependencies
- [`test-assertj-fluent-assertions`](rules/test-assertj-fluent-assertions.md) - Use AssertJ fluent assertions over raw JUnit asserts
- [`test-arrange-act-assert`](rules/test-arrange-act-assert.md) - Structure tests as arrange/act/assert
- [`test-descriptive-names`](rules/test-descriptive-names.md) - Write descriptive test names, not `test1`/`testFoo`
- [`test-nested-grouping`](rules/test-nested-grouping.md) - Group related tests with `@Nested`
- [`test-testinstance-lifecycle`](rules/test-testinstance-lifecycle.md) - Choose `@TestInstance` lifecycle deliberately
- [`test-avoid-logic-in-tests`](rules/test-avoid-logic-in-tests.md) - Avoid conditionals/loops inside test bodies
- [`test-one-concept-per-test`](rules/test-one-concept-per-test.md) - Assert one behavior/concept per test method
- [`test-junit5-extensions`](rules/test-junit5-extensions.md) - Use JUnit 5 extensions for cross-cutting test setup
- [`test-mock-boundaries-not-internals`](rules/test-mock-boundaries-not-internals.md) - Mock at architectural boundaries, not internal collaborators
- [`test-integration-test-separation`](rules/test-integration-test-separation.md) - Separate unit tests from integration tests

### 10. Documentation (MEDIUM)

- [`doc-javadoc-public-api`](rules/doc-javadoc-public-api.md) - Document all public API with Javadoc
- [`doc-javadoc-param-return-throws`](rules/doc-javadoc-param-return-throws.md) - Document `@param`/`@return`/`@throws`
- [`doc-package-info`](rules/doc-package-info.md) - Document packages with `package-info.java`
- [`doc-javadoc-since-deprecated`](rules/doc-javadoc-since-deprecated.md) - Use `@since` and `@deprecated` with migration guidance
- [`doc-javadoc-link-tags`](rules/doc-javadoc-link-tags.md) - Use `{@link}`/`{@code}` to cross-reference types and code
- [`doc-comment-why-not-what`](rules/doc-comment-why-not-what.md) - Write comments explaining why, not what the code already says
- [`doc-javadoc-code-samples`](rules/doc-javadoc-code-samples.md) - Include runnable code samples in Javadoc
- [`doc-module-info-documentation`](rules/doc-module-info-documentation.md) - Document JPMS modules in `module-info.java`
- [`doc-readme-module-level`](rules/doc-readme-module-level.md) - Maintain a README per module with purpose and usage

### 11. Performance Patterns (MEDIUM)

- [`perf-stringbuilder-loop-concat`](rules/perf-stringbuilder-loop-concat.md) - Use `StringBuilder` for string concatenation in loops
- [`perf-avoid-autoboxing-hot-path`](rules/perf-avoid-autoboxing-hot-path.md) - Avoid autoboxing primitives in hot paths
- [`perf-lazy-initialization-holder`](rules/perf-lazy-initialization-holder.md) - Defer expensive initialization with lazy holders
- [`perf-avoid-reflection-hot-path`](rules/perf-avoid-reflection-hot-path.md) - Avoid reflection in hot paths
- [`perf-string-intern-caution`](rules/perf-string-intern-caution.md) - Use `String.intern()` cautiously and deliberately
- [`perf-avoid-premature-optimization`](rules/perf-avoid-premature-optimization.md) - Don't optimize before profiling
- [`perf-jmh-benchmarking`](rules/perf-jmh-benchmarking.md) - Use JMH for reliable microbenchmarks
- [`perf-collection-sizing`](rules/perf-collection-sizing.md) - Size collections up front when the count is known
- [`perf-avoid-unnecessary-object-creation`](rules/perf-avoid-unnecessary-object-creation.md) - Avoid unnecessary object creation in hot paths
- [`perf-string-format-vs-concat`](rules/perf-string-format-vs-concat.md) - Choose `String.format`/concatenation deliberately by cost
- [`perf-primitive-arrays-hot-path`](rules/perf-primitive-arrays-hot-path.md) - Use primitive arrays for large numeric hot-path data
- [`perf-profile-before-optimizing`](rules/perf-profile-before-optimizing.md) - Profile before optimizing

### 12. Project Structure (LOW)

- [`proj-maven-gradle-standard-layout`](rules/proj-maven-gradle-standard-layout.md) - Follow the standard Maven/Gradle source layout
- [`proj-package-by-feature`](rules/proj-package-by-feature.md) - Organize packages by feature, not by technical layer
- [`proj-module-info-jpms`](rules/proj-module-info-jpms.md) - Weigh JPMS `module-info.java` trade-offs deliberately
- [`proj-multi-module-build`](rules/proj-multi-module-build.md) - Split a growing codebase into multiple Maven/Gradle modules
- [`proj-dependency-management-bom`](rules/proj-dependency-management-bom.md) - Centralize dependency versions with a BOM
- [`proj-package-private-default`](rules/proj-package-private-default.md) - Default new types/members to package-private
- [`proj-avoid-circular-package-deps`](rules/proj-avoid-circular-package-deps.md) - Avoid circular package dependencies
- [`proj-resources-separation`](rules/proj-resources-separation.md) - Separate resources from source under standard directories
- [`proj-gradle-version-catalog`](rules/proj-gradle-version-catalog.md) - Centralize versions in a Gradle version catalog
- [`proj-flat-small-projects`](rules/proj-flat-small-projects.md) - Keep small projects flat instead of over-modularizing

### 13. Linting & Static Analysis (LOW)

- [`lint-checkstyle-google-style`](rules/lint-checkstyle-google-style.md) - Enforce a Checkstyle ruleset (Google/Sun style) in CI
- [`lint-spotbugs-ci-gate`](rules/lint-spotbugs-ci-gate.md) - Run SpotBugs as a required CI gate
- [`lint-error-prone-compiler-plugin`](rules/lint-error-prone-compiler-plugin.md) - Compile with the Error Prone plugin
- [`lint-pmd-rulesets`](rules/lint-pmd-rulesets.md) - Run PMD rulesets for additional static analysis
- [`lint-nullaway-annotation-checking`](rules/lint-nullaway-annotation-checking.md) - Use NullAway for compile-time null-safety checks
- [`lint-suppress-with-justification`](rules/lint-suppress-with-justification.md) - Require a comment justification alongside any suppression
- [`lint-warnings-as-errors`](rules/lint-warnings-as-errors.md) - Treat compiler warnings as errors in CI
- [`lint-dependency-vulnerability-scan`](rules/lint-dependency-vulnerability-scan.md) - Scan dependencies for known vulnerabilities
- [`lint-jacoco-coverage-gate`](rules/lint-jacoco-coverage-gate.md) - Enforce a JaCoCo coverage gate in CI
- [`lint-editorconfig-formatting`](rules/lint-editorconfig-formatting.md) - Enforce formatting via `.editorconfig`/Spotless

### 14. Anti-patterns (REFERENCE)

- [`anti-return-null-instead-optional`](rules/anti-return-null-instead-optional.md) - Don't return `null` instead of `Optional`/empty collection
- [`anti-mutable-public-fields`](rules/anti-mutable-public-fields.md) - Don't expose mutable public fields
- [`anti-god-class`](rules/anti-god-class.md) - Don't build a God class with too many responsibilities
- [`anti-excessive-checked-exceptions`](rules/anti-excessive-checked-exceptions.md) - Don't force callers through excessive checked exceptions
- [`anti-string-concat-in-loop`](rules/anti-string-concat-in-loop.md) - Don't concatenate strings with `+` in a loop
- [`anti-catch-and-ignore`](rules/anti-catch-and-ignore.md) - Don't catch and silently ignore exceptions
- [`anti-raw-type-usage`](rules/anti-raw-type-usage.md) - Don't use raw generic types
- [`anti-instanceof-chain-instead-polymorphism`](rules/anti-instanceof-chain-instead-polymorphism.md) - Don't chain `instanceof` where polymorphism/pattern matching fits
- [`anti-singleton-static-abuse`](rules/anti-singleton-static-abuse.md) - Don't abuse the singleton pattern / static state
- [`anti-null-check-cascade`](rules/anti-null-check-cascade.md) - Don't cascade repetitive null checks instead of `Optional`/`Objects`
- [`anti-overuse-of-static`](rules/anti-overuse-of-static.md) - Don't overuse `static` methods and fields
- [`anti-exception-for-flow-control`](rules/anti-exception-for-flow-control.md) - Don't use exceptions for expected, routine flow control
- [`anti-blocking-call-on-virtual-thread-pinning`](rules/anti-blocking-call-on-virtual-thread-pinning.md) - Don't hold `synchronized` around blocking calls on virtual threads
- [`anti-magic-numbers-strings`](rules/anti-magic-numbers-strings.md) - Don't scatter unexplained magic numbers/strings
- [`anti-anemic-domain-model`](rules/anti-anemic-domain-model.md) - Don't reduce domain objects to anemic getter/setter bags

---

## Recommended Build Tooling

```xml
<!-- pom.xml (excerpt) -->
<properties>
    <maven.compiler.release>21</maven.compiler.release>
    <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
</properties>

<build>
    <plugins>
        <plugin>
            <groupId>org.apache.maven.plugins</groupId>
            <artifactId>maven-compiler-plugin</artifactId>
            <version>3.13.0</version>
            <configuration>
                <release>21</release>
                <compilerArgs>
                    <arg>-Xlint:all</arg>
                    <arg>-Werror</arg>
                    <arg>-XDcompilePolicy=simple</arg>
                    <arg>--should-stop=ifError=FLOW</arg>
                    <arg>-Xplugin:ErrorProne</arg>
                </compilerArgs>
                <annotationProcessorPaths>
                    <path>
                        <groupId>com.google.errorprone</groupId>
                        <artifactId>error_prone_core</artifactId>
                        <version>2.36.0</version>
                    </path>
                </annotationProcessorPaths>
            </configuration>
        </plugin>
        <plugin>
            <groupId>org.apache.maven.plugins</groupId>
            <artifactId>maven-checkstyle-plugin</artifactId>
            <version>3.6.0</version>
            <configuration>
                <configLocation>google_checks.xml</configLocation>
                <failOnViolation>true</failOnViolation>
            </configuration>
            <executions>
                <execution>
                    <phase>verify</phase>
                    <goals><goal>check</goal></goals>
                </execution>
            </executions>
        </plugin>
        <plugin>
            <groupId>com.github.spotbugs</groupId>
            <artifactId>spotbugs-maven-plugin</artifactId>
            <version>4.8.6.5</version>
            <configuration>
                <effort>Max</effort>
                <threshold>Medium</threshold>
                <failOnError>true</failOnError>
            </configuration>
        </plugin>
        <plugin>
            <groupId>org.jacoco</groupId>
            <artifactId>jacoco-maven-plugin</artifactId>
            <version>0.8.12</version>
            <executions>
                <execution>
                    <goals><goal>prepare-agent</goal></goals>
                </execution>
                <execution>
                    <id>check</id>
                    <goals><goal>check</goal></goals>
                    <configuration>
                        <rules>
                            <rule>
                                <limits>
                                    <limit>
                                        <counter>LINE</counter>
                                        <minimum>0.80</minimum>
                                    </limit>
                                </limits>
                            </rule>
                        </rules>
                    </configuration>
                </execution>
            </executions>
        </plugin>
    </plugins>
</build>
```

```kotlin
// build.gradle.kts (excerpt)
plugins {
    java
    checkstyle
    id("com.github.spotbugs") version "6.0.26"
    id("net.ltgt.errorprone") version "4.1.0"
    jacoco
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

tasks.withType<JavaCompile> {
    options.compilerArgs.addAll(listOf("-Xlint:all", "-Werror"))
    options.errorprone {
        disableWarningsInGeneratedCode.set(true)
    }
}

checkstyle {
    toolVersion = "10.20.2"
    configFile = file("$rootDir/config/checkstyle/google_checks.xml")
}

spotbugs {
    effort.set(com.github.spotbugs.snom.Effort.MAX)
    reportLevel.set(com.github.spotbugs.snom.Confidence.MEDIUM)
}

jacoco {
    toolVersion = "0.8.12"
}

tasks.test {
    useJUnitPlatform()
    finalizedBy(tasks.jacocoTestReport)
}

dependencies {
    errorprone("com.google.errorprone:error_prone_core:2.36.0")
    testImplementation(platform("org.junit:junit-bom:5.11.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testImplementation("org.mockito:mockito-junit-jupiter:5.14.2")
    testImplementation("org.assertj:assertj-core:3.26.3")
}
```

---

## How to Use

This skill provides rule identifiers for quick reference. When generating or reviewing Java code:

1. **Check relevant category** based on task type
2. **Apply rules** with matching prefix
3. **Prioritize** CRITICAL > HIGH > MEDIUM > LOW
4. **Read rule files** in `rules/` for detailed examples

### Rule Application by Task

| Task | Primary Categories |
|------|--------------------|
| New class/record | `api-`, `null-`, `name-` |
| New public API/library module | `api-`, `type-`, `doc-` |
| Concurrency/virtual threads | `conc-`, `modern-`, `err-` |
| Error handling | `err-`, `null-` |
| Modern language migration | `modern-`, `type-` |
| Collections/data pipelines | `coll-`, `perf-` |
| Performance tuning | `perf-`, `coll-`, `conc-` |
| Writing tests | `test-` |
| Project/module setup | `proj-`, `lint-` |
| Code review | `anti-`, `lint-` |

---

## Related Skills

- [design-patterns](../design-patterns/SKILL.md) - choosing and implementing GoF and idiomatic patterns; apply alongside `api-` rules here for pattern-heavy Java design (Builder, Factory, Strategy via sealed interfaces).
- [security-review](../security-review/SKILL.md) - security-focused audit checklists; apply alongside `err-`, `null-`, and `conc-` rules here when reviewing Java code for vulnerabilities (deserialization, injection, resource exhaustion).
- [kotlin-coding](../kotlin-coding/SKILL.md) - sibling JVM-language skill; consult its `interop-` category when a Java codebase interoperates with Kotlin (platform types, `@JvmStatic`/`@JvmOverloads`, nullability annotations at the boundary).

## Sources

This skill synthesizes best practices from:
- *Effective Java, 3rd Edition* (Joshua Bloch)
- The Java Language Specification (JLS), SE 21 and SE 25
- [Google Java Style Guide](https://google.github.io/styleguide/javaguide.html)
- Oracle's official Java coding conventions and OpenJDK JEP index (JEP 395, 409, 425/444, 431, 454, 453/480)
- [Error Prone](https://errorprone.info/bugpatterns) and [SpotBugs](https://spotbugs.readthedocs.io/en/stable/bugDescriptions.html) rule documentation
- Production codebases: Spring Framework, Guava, Netty, Apache Commons
- Community conventions (2024-2026)
