# Contributing

## Scope

Burp MCP is dual-use security software. Test only systems you own or are explicitly authorized to assess. Do not include credentials, private traffic, raw sensitive payloads, or customer data in issues, pull requests, fixtures, or logs.

## Development setup

Required toolchains:

- Rust 1.88.0
- Java 25
- Gradle 9.7.0

Build and verify the repository with:

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
gradle --no-daemon clean test jar
```

Run `scripts/run-grpc-interop.sh` when changing protobuf definitions, the Kotlin gRPC adapter, `burp-protocol`, or native transport behavior.

## Pull requests

- Branch from `main` and keep each pull request focused on one problem.
- Add or update tests only for changed observable behavior.
- Update documentation for user-visible tools, configuration, security boundaries, or release procedures.
- Keep `Cargo.toml`, `Cargo.lock`, `build.gradle.kts`, MCP server metadata, and JAR packaging expectations aligned when changing the version.
- Do not commit generated build output or local reference repositories.
- Complete the pull request template and identify checks that were not run.

CODEOWNERS review applies to all changes. Release, workflow, protocol, and transport changes require repository-owner review.

## Reporting vulnerabilities

Do not open a public issue for a suspected vulnerability. Follow [SECURITY.md](SECURITY.md).
