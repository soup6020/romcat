## 1. Baseline and Approval

- [x] 1.1 Confirm the user has explicitly approved edits to existing dependency, build, and narrowly required source files.
- [x] 1.2 Record the currently resolved direct dependency versions and baseline verification state.

## 2. Dependency Refresh

- [x] 2.1 Refresh `Cargo.lock` to the newest versions allowed by `Cargo.toml`, changing manifest constraints only if required, and review the dependency diff.
- [x] 2.2 Refresh the pinned `nixpkgs` revision in `flake.lock` and review the lockfile diff.

## 3. Compatibility and Quality Cleanup

- [x] 3.1 Apply Rust formatting and resolve dependency-related or pre-existing strict-Clippy failures with minimal behavior-preserving edits.
- [x] 3.2 Review all implementation and lockfile changes to confirm no CLI, parser-order, metadata, or I/O behavior was intentionally changed.

## 4. Verification

- [x] 4.1 Run `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo build` in the Nix development environment.
- [x] 4.2 Smoke-test help, version, and unknown-file handling.
- [x] 4.3 Run `nix build` and strict OpenSpec validation.
