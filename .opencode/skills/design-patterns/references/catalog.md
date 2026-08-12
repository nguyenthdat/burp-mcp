# Design Pattern Catalog (Index)

Use this index after establishing a concrete design pressure (see `SKILL.md`, "Establish the Pressure"). Each row links to the full rule file with per-language implementations, native-construct alternatives, and pitfalls. This page only summarizes when to reach for each pattern — do not implement from this table alone.

## Creational Patterns

| Pattern | Use When | First, Try |
|---|---|---|
| [Factory Method](../rules/factory-method.md) | Shared workflow must defer creation of one product to an implementation/subclass/function. | A plain constructor function, `new`, or `TryFrom`/parse function. |
| [Abstract Factory](../rules/abstract-factory.md) | A client must create compatible families of related products without naming concrete families. | A single factory function if there is only one product family in practice. |
| [Builder](../rules/builder.md) | Construction has many optional fields, ordered steps, validation, or multiple outputs from one recipe. | Named/keyword constructor arguments or a plain constructor with defaults. |
| [Prototype](../rules/prototype.md) | Callers need independent copies without knowing concrete construction details. | The language's built-in copy/clone (`Clone`, `copy.deepcopy`, copy constructor) with documented deep/shallow semantics. |
| [Singleton](../rules/singleton.md) | A process-wide resource genuinely has one initialization or identity. | Dependency injection from the composition root; a one-time lazy-init primitive only for truly immutable state. |
| [Object Pool](../rules/object-pool.md) | Object construction/teardown is measurably expensive and reuse is safe. | Just allocate normally until profiling shows a real cost. |

## Structural Patterns

| Pattern | Use When | First, Try |
|---|---|---|
| [Adapter](../rules/adapter.md) | A foreign or legacy interface must satisfy a local consumer contract. | A conversion function or a thin wrapper type. |
| [Bridge](../rules/bridge.md) | Two dimensions of behavior must vary independently without a Cartesian product of types. | Two independent enums/parameters if both dimensions are closed and small. |
| [Composite](../rules/composite.md) | Leaves and containers need uniform recursive operations. | A flat list plus a grouping key if there is no real recursive nesting. |
| [Decorator](../rules/decorator.md) | Behavior must be layered around the same interface and wrapper order matters. | A single function that composes the extra behavior inline. |
| [Facade](../rules/facade.md) | Clients need a small workflow-oriented entry point over a complex subsystem. | A well-named module/function if the subsystem is already small. |
| [Flyweight](../rules/flyweight.md) | Many objects repeat large immutable intrinsic state and measurement shows memory pressure. | Just duplicate the data until profiling shows a real cost. |
| [Proxy](../rules/proxy.md) | Access to a service needs authorization, lazy loading, caching, rate limiting, retries, or remote indirection behind the same contract. | Inline checks at the call site if there is only one call site. |

## Behavioral Patterns

| Pattern | Use When | First, Try |
|---|---|---|
| [Chain of Responsibility](../rules/chain-of-responsibility.md) | A request passes through ordered handlers that may handle, transform, reject, or forward it. | A single function with early returns if the handler list is fixed and short. |
| [Command](../rules/command.md) | Operations need queuing, logging, remote execution, retries, undo, or heterogeneous history. | Call the function directly if there is no need to store, queue, or undo the request. |
| [Interpreter](../rules/interpreter.md) | A small, stable grammar needs repeated evaluation (rules, filters, expressions). | An existing expression/query library or a data-driven config instead of a bespoke grammar. |
| [Iterator](../rules/iterator.md) | A collection needs traversal without exposing representation. | The language's built-in iteration protocol (`Iterator`, generators, `for..of`, `range`). |
| [Mediator](../rules/mediator.md) | Components have chaotic pairwise dependencies and coordination belongs in one owner. | Direct calls between the two components if there are only two. |
| [Memento](../rules/memento.md) | State needs undo, checkpoints, crash recovery, or snapshots without exposing internals. | A full clone of public state if internals do not need hiding. |
| [Observer](../rules/observer.md) | Multiple consumers must react to events and subscription changes over time. | A direct callback parameter if there is exactly one consumer. |
| [State](../rules/state.md) | Behavior and valid operations depend on current state. | An enum plus exhaustive match if states are closed and stable. |
| [Strategy](../rules/strategy.md) | One algorithm varies independently from its caller. | A function/closure parameter if there is no shared state between calls. |
| [Template Method](../rules/template-method.md) | An invariant algorithm skeleton has a few controlled customization steps. | A free function accepting callback/strategy parameters for the varying steps. |
| [Visitor](../rules/visitor.md) | A stable set of element shapes needs many independently evolving operations. | An enum plus exhaustive match if the element set changes more often than the operations. |
| [Null Object](../rules/null-object.md) | Callers repeatedly branch on null/None/missing before doing anything useful. | An `Option`/optional type with combinators if callers already handle absence idiomatically. |
| [Specification](../rules/specification.md) | A business predicate must be named, tested, and composed (AND/OR/NOT) independently of where it's used. | A plain boolean function or lambda if it is not reused or combined elsewhere. |

## Architectural / Concurrency Patterns

| Pattern | Use When | First, Try |
|---|---|---|
| [Dependency Injection](../rules/dependency-injection.md) | A component's collaborators vary by environment, test, or configuration. | Pass the dependency as a constructor/function argument; skip a DI framework for small graphs. |
| [Repository](../rules/repository.md) | Domain logic must not depend on a specific persistence technology. | Call the ORM/driver directly if there is exactly one storage backend and no test-double need. |
| [Unit of Work](../rules/unit-of-work.md) | Multiple repository operations must commit or roll back together. | A single transaction scope/context manager if there is only one repository involved. |
| [CQRS](../rules/cqrs.md) | Read and write workloads have very different shapes, scaling, or consistency needs. | One model for both reads and writes until that model demonstrably strains under either load. |
| [Event Sourcing](../rules/event-sourcing.md) | The system needs a full audit trail, temporal queries, or event-driven integration as the source of truth. | Standard CRUD with an audit/history table if you don't need to rebuild state from events. |
| [Pipeline/Middleware](../rules/pipeline-middleware.md) | Requests or messages pass through an ordered, composable set of cross-cutting steps. | A single function that inlines the steps if the set is fixed and short. |
| [Ports and Adapters](../rules/ports-and-adapters.md) | Domain logic must stay testable and swappable independent of frameworks/infrastructure. | A thin service layer if there is only one infrastructure integration and no swap requirement. |
| [Pub/Sub](../rules/pub-sub.md) | Producers and consumers must be decoupled, possibly across processes/services. | An in-process observer/event emitter if everything runs in one process. |
| [Circuit Breaker](../rules/circuit-breaker.md) | A downstream dependency fails intermittently and repeated calls make it worse. | A timeout plus retry-with-backoff if failures are rare and short-lived. |
