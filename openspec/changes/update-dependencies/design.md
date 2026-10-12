## Context

See `proposal.md` for motivation. The manifest uses broad compatible ranges (`clap = "4"`, `comfy-table = "7"`, `md-5 = "0.10"`), so refreshing `Cargo.lock` is the primary Rust dependency update mechanism unless upstream compatibility requires a manifest change. The Nix flake separately pins `nixpkgs` in `flake.lock`. The current source builds tests but does not pass the repository's documented formatting and strict-Clippy checks.

## Goals / Non-Goals

**Goals:**
- Resolve the newest dependency versions allowed by the existing compatibility ranges.
- Refresh the Nix input while preserving the current four-platform package/dev-shell structure.
- Produce lockfiles that build reproducibly.
- Finish with clean formatting, tests, strict Clippy, Cargo build, and Nix build.

**Non-Goals:**
- Widen major-version compatibility ranges without a demonstrated need.
- Add or remove runtime dependencies.
- Alter parser behavior, parser order, metadata, CLI flags, or table semantics.
- Fix unrelated parser caveats.

## Decisions

### Refresh compatible versions before changing manifest constraints

Run Cargo's lockfile update against the existing semver ranges. Change `Cargo.toml` only if a dependency's maintained release requires a deliberate constraint change.

Alternative considered: pin exact crate versions in `Cargo.toml`. Rejected because `Cargo.lock` already provides reproducibility for this binary crate while broad compatible ranges permit normal patch/minor maintenance.

### Refresh the pinned Nix input independently

Update `flake.lock` using the existing `nixpkgs-unstable` input and keep `flake.nix` structurally unchanged unless the new input exposes a build incompatibility.

Alternative considered: leave Nix pinned while only updating Cargo. Rejected because the request covers the program's dependency set and the flake is its documented build environment.

### Allow only verification-driven source cleanup

Apply `cargo fmt` and resolve strict-Clippy findings with behavior-preserving edits. Do not opportunistically refactor parser logic.

Alternative considered: accept a successful `cargo build` despite formatting/lint failures. Rejected because project context explicitly defines formatting and strict Clippy as completion checks.

### Verify through both toolchains

Use an isolated writable `CARGO_HOME`/`TMPDIR` when necessary in this environment, then run Cargo checks from `nix develop` and a final `nix build` using the committed lockfiles.

## Risks / Trade-offs

- [Upstream update changes transitive behavior] → Review the lockfile diff and run tests, strict Clippy, Cargo build, and a CLI smoke test.
- [Latest `nixpkgs-unstable` introduces toolchain churn] → Keep `flake.nix` unchanged where possible and verify `nix build`; rollback `flake.lock` independently if necessary.
- [Lint cleanup accidentally changes behavior] → Keep edits mechanical and review source diffs before verification.
- [No substantive automated regression suite exists] → Limit scope to dependency/build maintenance and perform a basic help/version/unknown-file smoke test in addition to compilation.

## Migration Plan

1. Record current resolved direct and relevant transitive versions.
2. Refresh `Cargo.lock` within existing manifest constraints and inspect the diff.
3. Refresh `flake.lock` and inspect the pinned revision change.
4. Apply only required formatting or strict-Clippy fixes.
5. Run all verification and smoke checks.
6. If verification cannot be restored without behavior changes, roll back the problematic lockfile update and report the blocker.
