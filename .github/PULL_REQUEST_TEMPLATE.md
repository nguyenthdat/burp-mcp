## Problem

Describe the user-visible problem or release requirement.

## Changes

- 

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace --locked`
- [ ] `cargo test --workspace --locked`
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- [ ] `gradle --no-daemon clean test jar`
- [ ] `scripts/run-grpc-interop.sh` when the protobuf, transport, Kotlin adapter, or native client changes

List any checks not run and why.

## Security and compatibility

- [ ] No secrets, credentials, raw sensitive traffic, or parameter values were committed.
- [ ] New high-impact behavior is bounded and documented.
- [ ] Protocol and configuration compatibility changes are documented.
- [ ] Release metadata is synchronized when this PR changes the version.
