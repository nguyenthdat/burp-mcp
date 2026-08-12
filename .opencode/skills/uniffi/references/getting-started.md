# Getting Started

Use these steps to set up a new UniFFI project from scratch.

## Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
uniffi = { version = "0.28", features = ["cli"] }

[build-dependencies]
uniffi = { version = "0.28", features = ["build"] }
```

The `cdylib` crate type is required — it produces the shared library
(`.so`, `.dylib`, `.dll`) that Kotlin/Swift load at runtime.

## build.rs (proc-macro approach)

```rust
fn main() {
    uniffi::generate_scaffolding("src/<name>.udl").unwrap();
}
```

For the proc-macro approach, `build.rs` can be minimal or omitted entirely if
using `uniffi::setup_scaffolding!()` in `lib.rs`.

## lib.rs — minimal proc-macro scaffold

```rust
uniffi::setup_scaffolding!();

#[uniffi::export]
fn hello(name: String) -> String {
    format!("Hello, {name}!")
}
```

`setup_scaffolding!()` must be called exactly once in the crate. It generates
the C FFI entry points that `uniffi-bindgen` uses to produce foreign bindings.

## lib.rs — UDL approach

```rust
uniffi::include_scaffolding!("<name>");
```

With UDL, create a `.udl` file then call `include_scaffolding!` with the
namespace defined in the UDL.

## Build and verify

```bash
cargo build --release
cargo test
ls target/release/lib<name>.so     # Linux
ls target/release/lib<name>.dylib  # macOS
```

## Generate foreign bindings

From UDL file:

```bash
cargo run --bin uniffi-bindgen generate src/<name>.udl --language kotlin
cargo run --bin uniffi-bindgen generate src/<name>.udl --language swift
```

From compiled library (preferred for multi-crate):

```bash
cargo build --release
cargo run --bin uniffi-bindgen generate \
  --library target/release/lib<name>.dylib \
  --language kotlin \
  --out-dir bindings/kotlin
```

## Install uniffi-bindgen globally (optional)

```bash
cargo install uniffi_bindgen
uniffi-bindgen --version
```

Installing globally avoids `cargo run --bin uniffi-bindgen` on every
invocation. Use `uniffi-bindgen` directly in scripts and CI.

## Project structure (recommended)

```
my-component/
├── Cargo.toml
├── build.rs
├── src/
│   ├── lib.rs              # uniffi::setup_scaffolding!() + exports
│   ├── models.rs           # #[derive(uniffi::Record)]
│   ├── errors.rs           # #[derive(uniffi::Error)]
│   └── my_component.udl    # Only if using UDL
├── bindings/
│   ├── kotlin/
│   │   └── my_component.kt
│   └── swift/
│       └── my_component.swift
├── uniffi.toml             # Optional: custom type mappings, external packages
└── tests/
    └── test_generated.rs   # Rust-side tests for generated bindings
```

## uniffi.toml (optional)

```toml
[bindings.kotlin]
package_name = "com.example.mycomponent"
cdylib_name = "my_component"

[bindings.swift]
cdylib_name = "my_component"
```

Configure `package_name` for Kotlin and `cdylib_name` for both Kotlin and
Swift to match your library's filename.
