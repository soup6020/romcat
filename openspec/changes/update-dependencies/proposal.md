## Why

romcat's dependency lockfile and Nix input should be refreshed so the program builds against current compatible releases and receives available maintenance and security fixes. This also provides an opportunity to establish a clean, reproducible build baseline.

## What Changes

### Goals
- Update Rust dependencies to current compatible stable releases and regenerate `Cargo.lock`.
- Refresh the pinned `nixpkgs` input in `flake.lock` when a newer revision is available.
- Make only minimal compatibility or lint/format adjustments needed for successful verification.
- Verify formatting, tests, strict Clippy, Cargo build, and Nix build.

### Non-goals
- Change ROM detection, metadata extraction, CLI behavior, output formatting, or parser order.
- Add new runtime capabilities or replace in-repository format parsing with third-party parsers.
- Perform unrelated refactoring or address known parser correctness caveats.

## Capabilities

### New Capabilities

None. This is dependency and build maintenance only.

### Modified Capabilities

None. Existing observable behavior remains unchanged, so this change opts out of delta specs.

## Impact

- Affected dependency/build files: `Cargo.toml`, `Cargo.lock`, and potentially `flake.lock`.
- Existing Rust source may receive narrowly scoped compatibility, formatting, or warning fixes only if required for clean verification; the user's request explicitly authorizes this maintenance work.
- All supported formats and CLI behavior are intended to remain compatible.
- Parser dispatch order, bounded header reads, and streaming MD5 behavior are unaffected.
- Runtime performance should remain materially unchanged apart from upstream dependency improvements.
