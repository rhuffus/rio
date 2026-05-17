# Development

## Requirements

- **Rust toolchain** (stable, edition 2024 features). Install via [rustup](https://rustup.rs/):
  ```sh
  brew install rustup-init
  rustup-init -y --default-toolchain stable
  . "$HOME/.cargo/env"
  ```
- **C compiler** for building `tree-sitter-bash`'s C parser. On macOS: Xcode CLI tools (`xcode-select --install`). On Linux: `gcc` from your distro.

## Build, test, lint

```sh
cargo build              # debug build
cargo build --release    # optimized build
cargo test               # all unit + integration tests
cargo fmt --all          # apply formatting
cargo clippy --all-targets --all-features -- -D warnings   # lint (CI fails on warnings)
```

Run `cargo fmt --check` and the `clippy` command above before opening a PR — they're what CI runs.

## Repository layout

```
rio/
├── src/
│   ├── main.rs           # CLI entry point: Cli::parse().run()
│   ├── lib.rs            # Module re-exports
│   ├── cli.rs            # clap derive + dispatch + resolve_content helper
│   ├── cli/{init,status,apply,diff}.rs
│   ├── block.rs          # Managed block markers + find / upsert / remove
│   ├── sidecar.rs        # `.rio` TOML format + hash_block helper
│   ├── parser.rs
│   ├── parser/bash.rs    # tree-sitter-bash wrapper, stanza classification
│   └── reconcile.rs      # (stub; lands in v0.6)
├── tests/cli.rs          # End-to-end integration tests over tempfile dirs
├── docs/
│   ├── ARCHITECTURE.md
│   ├── ROADMAP.md
│   └── DEVELOPMENT.md
└── .github/workflows/
    ├── ci.yml            # fmt + clippy + test on PR and push to main
    └── release.yml       # Cross-compile binaries on tag push (v*)
```

## Branch model

- `main` is protected. No direct pushes.
- All changes go through a PR against `main`.
- The `Test` status check must be green before merge.
- Solo-dev defaults: 0 required reviews, admin (repo owner) can override the protection in genuine emergencies.
- Squash-merge is preferred for a linear, readable history.

## Commit conventions

[Conventional Commits](https://www.conventionalcommits.org/) style:

- `feat(scope): summary` — new functionality
- `fix(scope): summary` — bug fix
- `test(scope): summary` — test additions or changes
- `refactor(scope): summary` — internal restructuring with no behavior change
- `chore: summary` — non-functional changes (deps, tooling)
- `ci: summary` — CI/CD changes
- `docs: summary` — documentation only

The scope is optional but recommended when the change touches a specific module (`block`, `sidecar`, `cli`, etc.).

## Releases

Tagging a release triggers the `release.yml` workflow:

```sh
git tag v<X.Y.Z>
git push --tags
```

The workflow:

1. Builds the binary in parallel for four targets: darwin-arm64, darwin-x86_64, linux-x86_64, linux-arm64.
2. Packages each as `rio-<os>-<arch>.tar.gz` and computes a `.sha256`.
3. Creates a GitHub Release with auto-generated notes from PRs since the previous tag.
4. The Homebrew tap (`rhuffus/homebrew-rio`) consumes these tarballs.

### Validating a release-workflow change

Use a throwaway tag (`v0.0.1-test`, `v0.0.0-rc1`, etc.), watch the run with `gh run watch`, then clean up:

```sh
git tag -d v0.0.1-test                               # delete locally
gh release delete v0.0.1-test --cleanup-tag --yes    # delete on remote, both release and tag
```

This was used during initial setup to validate the matrix cross-compile and the artifact upload.
