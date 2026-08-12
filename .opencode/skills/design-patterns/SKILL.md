---
name: design-patterns
description: "Language-agnostic design-pattern selection and implementation for architecture, refactors, reusable abstractions, runtime polymorphism, factories/builders, wrappers, state machines, commands, observers, and other GoF and modern architectural patterns. Applies whenever a non-trivial design in Rust, TypeScript, Python, Go, C#, Kotlin, C, C++, Swift, or any other language needs a pattern decision, or when reviewing pattern-heavy code. Prefer idiomatic native constructs and simpler concrete code first; do not use for naming a pattern without real design pressure."
compatibility: opencode
metadata:
  domain: cross-language
  audience: senior-developer
  workflow: architecture-implementation-review
---

# Design Patterns

Use design patterns as a vocabulary for recurring design pressures, not as a target architecture. Prefer the smallest construct the target language offers that makes the required variation, ownership, and invariants explicit. A pattern name is a communication shortcut for a design already justified by pressure — never the goal itself.

This skill is language-agnostic. Pair it with the relevant per-language coding-standards skill (`rust-coding`, `typescript-coding`, `python-coding`, `go-coding`, `csharp-coding`, `kotlin-coding`, `c-coding`, `cpp-coding`, `swift-coding`, etc.) whenever implementation or code review is in scope, so demonstration code gets normalized into that language's production idiom (typed errors instead of stringly-typed failures, no unnecessary cloning/copying, no unsafe global state, proper resource/memory ownership). For architecture-only advice, apply the decision process here without requiring code-quality commands that have no target repository.

Refactoring.Guru-style examples are pedagogical. Preserve their intent, but hold every language implementation to that language's current idiom, not a literal transliteration of a C++/Java original.

## Required Workflow

### 1. Establish the Pressure

Inspect the current code and identify:

- The concrete duplication, coupling, unstable dependency, runtime variation, state transition, or lifecycle problem.
- Which axis is expected to change and which must remain stable.
- Whether variants are closed and known at compile/author time or open and selected at runtime.
- Ownership, lifetime, thread-safety, cancellation, persistence, and compatibility (semver/API) constraints.
- A concrete second implementation or near-term requirement. Keep one-off private code concrete unless abstraction clearly reduces risk.

If no recurring pressure exists, do not add a pattern. Record the simpler alternative and continue with concrete code.

### 2. Try Native Constructs First

Evaluate the language's own idioms before introducing a custom class/trait/protocol hierarchy:

| Requirement | Rust | TypeScript | Python | Go | C# | Kotlin | C | C++ | Swift |
|---|---|---|---|---|---|---|---|---|---|
| Closed set of variants | `enum` + exhaustive `match` | discriminated union + exhaustive `switch` (`never` check) | `Enum` + `match` statement, or `@dataclass` subclasses | `iota`-based typed const + `switch` on a type/tag | sealed `record`/class + pattern-matching `switch` | `sealed class`/`sealed interface` + `when` | tagged `struct` (enum tag + union payload) + `switch` | `std::variant` + `std::visit`, or a tagged struct + `switch` | `enum` with associated values + `switch` |
| Stateless behavior injection | closure / `Fn` bound | function value / arrow function | function or `functools.partial` | function value (`func` type) | delegate / `Func<T>` / lambda | function type / lambda | function pointer | `std::function` / lambda / template callable | closure |
| Compile-time polymorphism | generics / `impl Trait` / associated types | generics (structurally typed, erased) | `typing.Protocol` + duck typing (checked statically only) | generics (Go 1.18+) or implicit structural interfaces | generics with constraints (reified via JIT) | generics with bounded type parameters (JVM erasure) | `_Generic` (C11) or code generation/macros | templates / concepts (C++20) | generics with protocol conformances |
| Adaptation / validation wrapper | newtype wrapper | branded type / wrapper class | thin wrapper class or `NewType` | defined type with methods | wrapper `readonly struct`/class | value class / inline class | wrapper `struct` with named accessor functions | wrapper class / strong typedef | wrapper `struct` conforming to a protocol |
| Async/event coordination | channel or explicit event enum | `EventEmitter` / `Promise` / async iterator | `asyncio.Queue` / callback | channel (`chan`) + goroutine | `Channel<T>` / `event` / `IObservable<T>` | `Channel` / `Flow` / coroutines | callback + explicit state machine | `std::condition_variable`/queue or a reactor library | `AsyncStream` / Combine publisher |

Use a named pattern only when it communicates the design better than these constructs alone. Read `references/catalog.md` and the relevant `rules/<pattern-id>.md` for the pattern-specific decision detail and per-language code.

### 3. Choose the Dispatch Model

Use the least flexible model that satisfies the requirement, expressed in each language's own terms:

| Requirement | Rust | TypeScript | Python | Go | C# | Kotlin | C | C++ | Swift |
|---|---|---|---|---|---|---|---|---|---|
| Closed variants, variant-specific data | `enum` + `match` | discriminated union + `switch` | `Enum`/dataclass + `match` | typed const-iota + type switch | sealed `record` + `switch` expr | `sealed class` + `when` | tagged union `struct` + `switch` | `std::variant` + `std::visit` | `enum` w/ associated values + `switch` |
| Stateless replaceable behavior | closure/`Fn` | function value | function/callable | `func` value | delegate/`Func<T>` | function type/lambda | function pointer | `std::function`/lambda | closure |
| Compile-time composition, hot paths | generics/`impl Trait` | generics (erased at compile time) | `Protocol` (static-check only) | generics (Go 1.18+) | generics (JIT-reified) | generics (JVM erasure) | macros/inline functions | templates/concepts | generics (specialized) |
| Runtime choice among a closed set | `enum`/`match` | `switch` on discriminant | `match`/`isinstance` dispatch | type switch | `switch` expression/pattern match | `when` | `switch` on enum tag | `switch`/`std::visit` | `switch` |
| Open/stored heterogeneous runtime impls | `dyn Trait` | interface/abstract class instance | ABC subclass instance | interface value | interface/abstract class instance | interface/abstract class instance | struct-of-function-pointers ("manual vtable") | virtual base class pointer/reference | protocol existential (`any Protocol`) |
| Cross-task/cross-thread events | typed channel | message queue/worker events | queue/`asyncio` | channel | `Channel<T>`/message queue | `Channel`/actor | external message queue | thread-safe queue/condition variable | Combine/`AsyncStream` |

Before choosing dynamic dispatch (`dyn Trait`, interface reference, `any Protocol`, vtable struct), verify object/protocol safety, ownership, lifetime/retain semantics, thread-safety, allocation, and downcasting requirements. Before choosing generics/templates, assess compile time, binary/monomorphization size, and public API exposure.

Libraries should normally preserve a static generic API so callers retain the dispatch choice. An application that must store one runtime-selected implementation may erase the type at its composition boundary (`main`, DI container, app delegate) without forcing dynamic dispatch on every library consumer.

### 4. Design Ownership Explicitly

- Prefer composition and top-down ownership. Inheritance-shaped object graphs are rarely required to express a pattern's intent, in any language.
- Pass context into operations when it is short-lived; do not store self-referential or permanent references merely to imitate a class diagram.
- Avoid unnecessary shared mutable state as a default graph-building tool, regardless of whether the language is garbage-collected:
  - **Rust**: avoid `Rc<RefCell<_>>`/`Arc<Mutex<_>>` as a default; prefer ownership transfer, stable IDs, arenas, `Weak`, or channels. Use `OnceLock`/`LazyLock` for one-time immutable init, never `static mut`. Never hold a lock guard across `.await`.
  - **C++**: prefer value types and `unique_ptr` for sole ownership; reserve `shared_ptr` for genuine shared lifetime, and break cycles with `weak_ptr`. RAII owns cleanup; avoid raw owning pointers.
  - **C**: ownership is a naming/documentation contract (`_create`/`_destroy`, "caller frees"). Be explicit about who frees what; avoid ambient globals for anything test-sensitive.
  - **Swift**: ARC still leaks via retain cycles — use `weak`/`unowned` in closures and delegate references. Value types (`struct`/`enum`) avoid the problem entirely; prefer them for models.
  - **Go**: prefer passing values/interfaces over sharing pointers across goroutines; guard genuinely shared state with a `sync.Mutex` or a channel-owning goroutine, not both.
  - **C#/Kotlin/TypeScript/Python** (GC-managed): manual ownership ceremony is unnecessary, but unnecessary shared mutable singletons still cause the same problems — hidden coupling, hard-to-test code, and cross-request/cross-test state leakage. Prefer constructor/DI-provided instances scoped to a request, session, or test, not static/module-level mutable singletons.
- Avoid Singleton by default in every language. Prefer dependency injection from the composition root (`main`, DI container, app entry point). Where a language needs one-time lazy init (`OnceLock`/`LazyLock` in Rust, `lazy` in Kotlin, module-level constant in Python, `static let` in Swift, `sync.Once` in Go), use it only for genuinely immutable state, and never as a substitute for passing a dependency. These mechanisms are not reset hooks — keep resettable test state injected, or use process/test isolation when reset is required.
- Keep lock/mutex scope bounded in every language that has one (`Mutex`, `synchronized`, `lock`, `pthread_mutex_t`, GIL-adjacent `asyncio.Lock`). Define poison/panic, cancellation, reentrancy, and backpressure behavior where relevant.

### 5. Implement the Pattern Contract

- Keep interfaces/traits/protocols narrow and consumer-oriented. Seal or otherwise close public extension points when external implementations are not part of the contract.
- Preserve domain errors and return the language's typed-failure idiom — `Result<T, E>` (Rust), a typed `Either`/discriminated error union (TypeScript), a specific exception type (Python/Java/C#/Kotlin/Swift `throws`), an `error` return value (Go), or an explicit error/status output parameter (C) — for fallible construction, adaptation, proxying, undo, persistence, and notification. Do not collapse failures into a boolean, a string, or a silently caught exception.
- Make ordering, short-circuiting, retries, cache invalidation, subscription lifetime, and state transitions explicit.
- Mark discardable-by-mistake return values so misuse is caught: `#[must_use]` (Rust), `[[nodiscard]]` (C++), a linter rule for unused Promises/results (TypeScript), unused-return warnings (Go vet, Kotlin `@CheckReturnValue`, `-Wunused-result` in C/C++).
- Do not add a dependency, framework, or DI container only to obtain a pattern name. Prefer the standard library and existing project abstractions.
- Do not expose the pattern vocabulary in public names unless it helps users understand the API (`FooVisitor`, `FooFactory` are fine when they clarify; `FooManager`/`FooHelper` rarely do).

### 6. Record the Decision

For every introduced or materially changed pattern, add this to the architecture or implementation artifact. For read-only advice without an artifact, return the same decision record in the response:

```text
Pressure: <specific problem and axis of change>
Decision: <pattern or simpler native construct>
Language form: <enum/union | closure | generic/template | interface/protocol/trait object | channel/queue | wrapper>
Ownership: <who owns state and how references/messages/values flow>
Alternatives: <at least one rejected option and why>
Costs: <allocation, dispatch, compile/build time, binary size, locking, API/compatibility>
Invariants: <behavior that tests must enforce>
```

Naming a pattern without these fields is not a design decision.

## Pattern Catalog

Every entry below links to `rules/<pattern-id>.md`, which contains the full Intent & Pressure discussion, native-construct alternative, and one idiomatic code example per language (Rust, TypeScript, Python, Go, C#, Kotlin, C, C++, Swift).

### Creational

- [factory-method](rules/factory-method.md) — defer creation of one product to a subclass/implementation/function.
- [abstract-factory](rules/abstract-factory.md) — create families of related products without naming concrete types.
- [builder](rules/builder.md) — construct a complex value through optional/ordered steps with validation.
- [prototype](rules/prototype.md) — copy an existing object instead of rebuilding it from scratch.
- [singleton](rules/singleton.md) — guarantee (and usually avoid) a single process-wide instance.
- [object-pool](rules/object-pool.md) — reuse expensive-to-create objects instead of reallocating them.

### Structural

- [adapter](rules/adapter.md) — make a foreign interface satisfy a local contract.
- [bridge](rules/bridge.md) — vary abstraction and implementation independently.
- [composite](rules/composite.md) — treat individual objects and compositions of objects uniformly.
- [decorator](rules/decorator.md) — layer behavior around a shared interface, order-sensitively.
- [facade](rules/facade.md) — expose a small workflow-oriented entry point over a complex subsystem.
- [flyweight](rules/flyweight.md) — share immutable intrinsic state across many logical instances.
- [proxy](rules/proxy.md) — control access to a service behind its own interface.

### Behavioral

- [chain-of-responsibility](rules/chain-of-responsibility.md) — pass a request through ordered handlers until one handles it.
- [command](rules/command.md) — turn a request into an object/value that can be queued, logged, or undone.
- [interpreter](rules/interpreter.md) — represent a small grammar and evaluate sentences in it.
- [iterator](rules/iterator.md) — traverse a collection without exposing its representation.
- [mediator](rules/mediator.md) — centralize chaotic peer-to-peer coordination in one owner.
- [memento](rules/memento.md) — capture and restore state without exposing internals.
- [observer](rules/observer.md) — notify dependents of state changes without tight coupling.
- [state](rules/state.md) — let behavior change with an object's internal state.
- [strategy](rules/strategy.md) — vary an algorithm independently of the code that uses it.
- [template-method](rules/template-method.md) — fix an algorithm's skeleton while letting steps vary.
- [visitor](rules/visitor.md) — add operations to a stable set of element types without modifying them.
- [null-object](rules/null-object.md) — replace null/None/undefined checks with a neutral, do-nothing implementation.
- [specification](rules/specification.md) — encapsulate and compose a business predicate as a first-class value.

### Architectural / Concurrency

- [dependency-injection](rules/dependency-injection.md) — supply collaborators from outside instead of constructing them internally.
- [repository](rules/repository.md) — abstract persistence behind a collection-like domain interface.
- [unit-of-work](rules/unit-of-work.md) — track and commit a batch of changes as one transaction.
- [cqrs](rules/cqrs.md) — separate the read model from the write model.
- [event-sourcing](rules/event-sourcing.md) — persist state as an append-only sequence of events.
- [pipeline-middleware](rules/pipeline-middleware.md) — compose ordered request/response processing stages.
- [ports-and-adapters](rules/ports-and-adapters.md) — isolate domain logic from infrastructure behind ports (hexagonal architecture).
- [pub-sub](rules/pub-sub.md) — broadcast events to decoupled, possibly-distributed subscribers.
- [circuit-breaker](rules/circuit-breaker.md) — stop calling a failing dependency to protect the caller and let it recover.

## Review Rules

Treat these as BLOCKER when they can affect correctness, soundness, or public compatibility; otherwise report them as WARNING:

- A pattern solves no demonstrated pressure or duplicates a simpler native construct.
- Dynamic dispatch, shared mutability, or allocation/heap indirection was introduced without a runtime requirement.
- Ownership cycles, lock/mutex scope, callback reentrancy, cancellation, or thread-safety are unspecified.
- Pattern-specific semantics are broken, such as a proxy bypassing policy, a chain continuing after handling, a state machine permitting invalid transitions, or an event-sourced aggregate replaying non-deterministically.
- Public interfaces/traits/protocols expose accidental implementation details or create an unnecessary compatibility commitment.
- A Singleton hides a dependency, blocks test isolation, or uses unsynchronized mutable global state.

Do not block merely because code does not use a named pattern. Concrete code is preferred when it is simpler and sufficient.

## Verification

Run the relevant per-language coding-standards gates and add tests for the selected pattern's contract:

- Construction: required fields, incompatible families, validation failures, and copy/clone depth.
- Pluggable algorithms/backends: one conformance suite across implementations, correct runtime selection, and static callers remaining free of forced type erasure.
- Wrappers: transparent behavior, error propagation, policy enforcement, and wrapper ordering.
- Trees and chains: traversal/order, short-circuit behavior, empty cases, and depth/resource bounds.
- Commands and snapshots: idempotency where required, undo/redo, failed execution, and versioned restore.
- Events: subscribe/unsubscribe lifetime, reentrancy, slow consumers, backpressure, and delivery guarantees.
- State: allowed and rejected transitions, exhaustiveness, persistence, and concurrency behavior.
- Process-wide resources: no unsynchronized global mutation, explicit initialization behavior, and isolated parallel tests without cross-test state leakage.
- Architectural patterns (CQRS, event sourcing, ports-and-adapters, pub-sub, circuit-breaker): boundary contracts, replay determinism, timeout/half-open transitions, and infrastructure-swap tests (in-memory adapter vs. real adapter).

Use property tests for stable invariants and transition systems when the state space justifies them. Benchmark only when the pattern choice includes a performance claim.

## References

- `references/catalog.md` — one-page index of all patterns by category, for quick lookup before opening a specific rule file.
- `rules/<pattern-id>.md` — full Intent & Pressure, native-construct alternative, per-language implementation, and pitfalls for each pattern.
- [Refactoring.Guru: Design Patterns](https://refactoring.guru/design-patterns) — source catalog with parallel examples in C++, C#, Go, Java, PHP, Python, Ruby, Rust, Swift, and TypeScript.
- Gang of Four, *Design Patterns: Elements of Reusable Object-Oriented Software* (Gamma, Helm, Johnson, Vlissides) — the canonical creational/structural/behavioral catalog.
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) — Rust-specific idioms and anti-patterns.
- Joshua Bloch, *Effective Java* — idiomatic builder, singleton, and immutability guidance that also underlies most Kotlin API design.
- Kotlin language documentation, "Idioms" — `sealed class`/`when`, `object`, scope functions as pattern building blocks.
- Brandon Rhodes / Python Patterns (`python-patterns.guide`) and the standard library `abc`/`typing.Protocol` docs — Python-idiomatic pattern implementations.
- Go proverbs ("accept interfaces, return structs") and the standard library (`io.Reader`/`sort.Interface`) as the primary source of Go-idiomatic pattern shapes.
- Eric Evans, *Domain-Driven Design* — source for repository, unit-of-work, specification, and ports-and-adapters framing.
- Martin Fowler, *Patterns of Enterprise Application Architecture*, and martinfowler.com articles on CQRS and event sourcing.
