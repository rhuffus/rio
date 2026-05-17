# Architecture

This document describes the design model of `rio` and the architectural decisions that have been **locked** — settled during initial design and not up for reopening without explicit signal.

## The model: block-based co-residency

A `rio`-managed file is **not** a symlink, a generated artifact, or a template render. It is a real file owned by the user. Inside, a delimited block is owned by `rio`:

```
<user content above>

# >>> RhuffusIO Managed Block >>>
<rio-managed content>
# <<< RhuffusIO Managed Block <<<

<user content below>
```

The block delimiters are constants in `src/block.rs`:

- `BLOCK_OPEN` = `# >>> RhuffusIO Managed Block >>>`
- `BLOCK_CLOSE` = `# <<< RhuffusIO Managed Block <<<`

This pattern is borrowed from `conda init`, `asdf`, `nvm` and similar tools. It enables three things:

1. The user's personal edits coexist with rio's managed content in the same file.
2. Drift detection is well-defined: content inside the markers belongs to rio, outside belongs to the user.
3. Removal is non-destructive: `rio remove` strips only the block, preserving everything else.

## Sidecar state: `.rio`

Each managed file at `<path>` has a sibling sidecar at `<path>.rio` (e.g., `~/.zshrc.rio`). It is TOML and intended to be versioned in git for portability across machines.

Schema v1 (see `src/sidecar.rs`):

```toml
version = 1
managed_block_hash = "<sha256 of current block content>"
ignored = ["<hash1>", "<hash2>", ...]
```

The sidecar is the source of truth for two things:

- **What rio expects this file's block to contain** (via `managed_block_hash`) — used for drift detection.
- **Which stanzas the user has explicitly chosen to ignore** (v0.6 reconciliation) — used to avoid re-prompting.

### Why a per-file sidecar (not a central state DB)?

`chezmoi` uses a central BoltDB at `~/.config/chezmoi/chezmoistate.boltdb`. `rio` instead uses a per-file sidecar because:

- Sidecars are diff-able in code review and obvious in directory listings.
- Sidecars are portable across machines (commit + pull).
- Sidecars are co-located with their file, making the ownership relationship visible at a glance.
- A bug or stale lock in central state can hose all your dotfiles; with sidecars, blast radius is one file.

## Semantic comparison: tree-sitter

To decide whether two stanzas are "the same configuration" with different syntax (e.g., `FOO=bar` outside the block vs `export FOO=bar` inside), `rio` parses each side and compares AST nodes — not strings.

v0.1 ships `tree-sitter-bash`. Future versions add INI (gitconfig), ssh-config, JSON, YAML, and TOML.

### Why not an LLM?

An LLM-first comparison engine was considered and rejected. It would introduce:

- **Latency** (seconds per reconcile vs sub-millisecond for tree-sitter).
- **Cost** (per-call API charges, ongoing).
- **Network dependency** (no offline operation).
- **API key management** chicken-and-egg: rio's purpose is to manage secrets, but the LLM API needs a secret.
- **Privacy concerns** (shell configs and infra hostnames sent to a third-party endpoint).
- **Non-determinism** (the same input may yield different decisions across runs).

Tree-sitter is deterministic, offline, sub-millisecond, free, and well-supported. If a comparison is ambiguous, `rio` asks the user — not an LLM.

## Build vs adopt: why not `chezmoi` or `yadm`?

These were evaluated against `rio`'s scope:

- **chezmoi** covers ~50%: per-host via Go templates, native 1Password functions, `modify_` scripts as escape hatch. **Missing**: block co-residency (replaces files wholesale), AST-level merge of structured files, interactive drift reconciliation, per-file sidecar state.
- **yadm** covers ~35%: alternates by hostname/OS/class. Simpler model, less powerful. **Missing**: native 1Password, all four differentiators.

The four differentiators below cannot be retrofitted cleanly via `chezmoi`'s `modify_` script mechanism without losing unified UX:

1. Block-delimited managed regions co-resident with user content.
2. AST-level merge of structured files (JSON / YAML / TOML).
3. Interactive drift reconciliation with promote / ignore / delete.
4. `.rio` sidecar per file persisting user decisions.

Hence: build, don't adopt.

## Locked decisions

These should not be reopened without explicit signal:

| Decision | Choice |
|----------|--------|
| Language | Rust (edition 2024) |
| Binary distribution | Homebrew tap `rhuffus/rio` |
| Comparison engine | tree-sitter only (no LLM) |
| State storage | per-file `.rio` sidecar, TOML, versioned in git |
| Block markers | `# >>> RhuffusIO Managed Block >>>` / `# <<< RhuffusIO Managed Block <<<` |
| Stanza granularity for prompts | separated by blank lines |
| Per-host layering *(v0.4)* | `hostname -s` autodetected; `managed/base/` + `managed/hosts/<host>/`, host wins |
| Secrets *(v0.5)* | 1Password Environments; local = mounted `.env` (biometric 1× per unlock); CI/CD = Service Account + `op inject` |
| License | Apache-2.0 |
| Naming | `rio` (acronym RhuffusIO); same name for binary, repo, crate, tap |
| Cross-compile targets | darwin-arm64, darwin-x86_64, linux-x86_64, linux-arm64 |
| Branch model | `main` protected, PR-required, `Test` status check must pass, 0 reviews (solo dev), admin override allowed |
