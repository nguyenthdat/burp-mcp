# Rust Module Structure

These rules are mandatory when creating or reorganizing Rust production and test code.

## Organize Multi-File Features As Directory Modules

- A concept implemented by one small, cohesive source file may stay flat, for example `src/tlsplus/engine.rs`.
- As soon as that concept requires a second concept-specific production source file, convert it to a directory module. Do not create prefixed sibling files to simulate a namespace.
- Use `mod.rs` as the feature entry point and give child files short responsibility-based names.

Required layout for a multi-file `engine` feature:

```text
src/tlsplus/
|-- mod.rs
`-- engine/
    |-- mod.rs
    |-- api.rs
    |-- lifecycle.rs
    |-- runtime.rs
    |-- state.rs
    `-- types.rs
```

Equivalent compact form:

```text
src/tlsplus/engine/{mod.rs,api.rs,lifecycle.rs,runtime.rs,state.rs,types.rs}
```

Forbidden flat prefix fan-out:

```text
src/tlsplus/engine_api.rs
src/tlsplus/engine_lifecycle.rs
src/tlsplus/engine_runtime.rs
src/tlsplus/engine_state.rs
src/tlsplus/engine_types.rs
```

Reject the same pattern for other concepts, such as `server_api.rs`, `server_runtime.rs`, and `server_types.rs`. Rust's module tree must express ownership and hierarchy; do not use JavaScript- or Go-style filename namespacing.

## Define A Clear Module Boundary

The parent module declares the feature once:

```rust
mod engine;

pub use engine::{TlsPlusEngine, TlsPlusEngineError};
```

The feature entry point owns its child modules and public surface:

```rust
mod api;
mod lifecycle;
mod runtime;
mod state;
mod types;

pub use api::TlsPlusEngine;
pub use types::TlsPlusEngineError;
```

- Keep implementation modules and items private by default.
- Use `pub(super)` for parent-only access and `pub(crate)` for crate-internal APIs.
- Use `pub` only for an intentional public API.
- Re-export the stable feature API from `mod.rs` so callers do not depend on internal paths such as `engine::api` or `engine::types`.
- Avoid wildcard re-exports unless the entire child module intentionally forms the public API.

## Keep Rust Tests In The Module Tree

- Keep a small unit-test set inline in the production module, behind `#[cfg(test)] mod tests { ... }`.
- When unit tests become large enough to need a separate file, convert the entire production module to a directory module. Move `auth.rs` to `auth/mod.rs`, then place the tests in `auth/tests.rs`.
- Use the neutral child-module name `tests.rs`, never a production-name prefix or a `_tests.rs` suffix.
- Declare the external unit-test module from the directory module's `mod.rs` with `#[cfg(test)] mod tests;` and import private implementation details in `tests.rs` with `use super::*;`.

For short tests, keep everything in one file:

```text
src/tlsplus/auth.rs
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests belong here while the test set remains small.
}
```

When tests need their own file, convert `auth` to a directory module:

```text
src/tlsplus/auth/
|-- mod.rs
`-- tests.rs
```

The declaration belongs in `auth/mod.rs`:

```rust
#[cfg(test)]
mod tests;
```

Forbidden test-file names and layouts:

```text
src/tlsplus/auth/auth_tests.rs
src/tlsplus/auth_tests.rs
src/tlsplus/engine/engine_tests.rs
src/tlsplus/auth.rs + src/tlsplus/auth/tests.rs
```

Never keep both `auth.rs` and an `auth/` directory for that module's unit tests. Once tests are split out, `auth/mod.rs` must replace `auth.rs`. Do not use `#[path = "auth/auth_tests.rs"]` or any custom path attribute to bypass this rule. Integration tests that exercise the public crate API belong in the crate-level `tests/` directory and should be named after the behavior or domain, such as `tests/auth.rs`, not `tests/auth_tests.rs`. Doctests remain next to the public item they document.

## Exceptions And Scope

- Integration tests, generated sources, platform-specific implementations, and unrelated neighboring modules do not by themselves trigger directory conversion.
- Do not split a cohesive one-file module merely to satisfy this rule.
- When code is already being added to or reorganized within a violating multi-file feature, normalize that feature to a directory module as part of the same change.
- Do not migrate unrelated legacy modules during an otherwise isolated task.
