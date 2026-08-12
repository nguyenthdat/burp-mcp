---
name: uniffi
description: >-
  UniFFI reference for generating Kotlin and Swift bindings from Rust
  libraries. Covers project setup (Cargo.toml, build.rs), both proc-macro
  (#[uniffi::export]) and UDL interface definition approaches, full Rust API
  surface (objects, records, enums, errors, constructors, async), built-in type
  mappings, Kotlin bindings generation and consumer code (Gradle, coroutines,
  Deferred), and Swift bindings generation and consumer code (Xcode,
  xcframework, module compilation). Use whenever the user mentions UniFFI,
  uniffi-rs, uniffi-bindgen, Rust FFI bindings, Rust to Kotlin, Rust to Swift,
  or asks to expose a Rust library to Kotlin/Swift, create cross-platform Rust
  bindings, or integrate Rust with Android/iOS. Prefer current docs at
  https://mozilla.github.io/uniffi-rs/ over memory for exact API.
compatibility: opencode
metadata:
  domain: rust-ffi
  audience: software-engineer
---

# UniFFI

Use this skill for creating multi-language bindings from Rust to Kotlin and
Swift. UniFFI generates a C-compatible FFI layer from Rust, then produces
idiomatic foreign language bindings.

## Working rules

- **Choose proc-macros for new projects.** `#[uniffi::export]` is the modern,
  ergonomic approach. Use UDL files only for legacy or complex cross-crate
  scenarios.
- **The Rust crate must produce a `cdylib`.** Set `crate-type = ["cdylib"]` in
  `Cargo.toml`. This is required for the shared library that Kotlin/Swift will
  load.
- **Call `uniffi::setup_scaffolding!()` in `lib.rs` or `build.rs`.**
  Proc-macros need this macro invoked once in the crate. UDL-based setups call
  `uniffi::include_scaffolding!()` instead.
- **Use `uniffi-bindgen` to generate foreign code.** Point it at the `.udl`
  file or the compiled `cdylib` (`--library` mode). Library mode is preferred
  for multi-crate projects.
- **Async Rust maps to Kotlin coroutines (`Deferred`) and Swift's
  `async/await`.** Requires `kotlinx-coroutines-core` on Android.
- **Type mappings are automatic:** `Vec<T>` → `List<T>` (Kotlin) / `[T]`
  (Swift), `HashMap<K,V>` → `Map<K,V>` / `Dictionary<K,V>`, `Option<T>` →
  `T?`, `String` → `String`, `Vec<u8>` → `ByteArray` / `Data`.
- **Check exact docs before complex types.** Custom types, external types, and
  callback interfaces have specific requirements. Refer to the official docs.

## First checks

```bash
cargo install uniffi_bindgen
uniffi-bindgen --version
cargo add uniffi --features cli
```

## Pipeline overview

```
Rust source (proc-macros or UDL)
       │
       ▼
  uniffi::setup_scaffolding!()
       │
       ▼
  cargo build --release          →  lib<name>.so / .dylib
       │
       ▼
  uniffi-bindgen generate        →  <name>.kt / <name>.swift
       │
       ▼
  Kotlin / Swift consumer code
```

## Pick the right reference

| Goal | Read |
|------|------|
| Project setup, Cargo.toml, build.rs, first scaffold | `references/getting-started.md` |
| Rust API: proc macros, UDL, types, objects, errors, async | `references/rust-api.md` |
| Kotlin: bindings generation, Gradle, consumer code, coroutines | `references/kotlin-bindings.md` |
| Swift: bindings generation, iOS integration, consumer code, module | `references/swift-bindings.md` |

Load only the relevant reference file before writing code.

## Rust → Kotlin / Swift type map

| Rust | Kotlin | Swift |
|------|--------|-------|
| `bool` | `Boolean` | `Bool` |
| `u8`–`u64`, `i8`–`i64` | `UByte`–`ULong`, `Byte`–`Long` | `UInt8`–`UInt64`, `Int8`–`Int64` |
| `f32`, `f64` | `Float`, `Double` | `Float`, `Double` |
| `String` | `String` | `String` |
| `Vec<T>` | `List<T>` | `[T]` |
| `HashMap<K,V>` | `Map<K,V>` | `Dictionary<K,V>` |
| `Option<T>` | `T?` | `T?` |
| `Vec<u8>` | `ByteArray` | `Data` |
| `()` | `Unit` | `Void` / `()` |
| `SystemTime` | `java.time.Instant` | `Date` |
| `Duration` | `java.time.Duration` | `TimeInterval` |
