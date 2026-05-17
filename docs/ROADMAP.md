# Roadmap

`rhio` is built in small, publishable increments. Each version represents a coherent slice of functionality with its own integration tests and is releasable on its own.

## v0.1.0 — MVP

**Status**: shipped (modulo the Homebrew tap).

### Shipped

- `block` module — managed-block markers, `find` / `upsert` / `remove` operations on text files. Detects malformed blocks (orphan markers, duplicates, close-before-open).
- `sidecar` module — TOML schema v1 with `version`, `managed_block_hash`, `ignored`. sha256-based stable hashing. Load/save roundtrip with anyhow context.
- `parser::bash` module — tree-sitter-bash wrapper. Top-level stanza classification: `Assignment{name}`, `Function{name}`, `Other`.
- CLI commands wired to the modules:
  - `rhio init <path> [--content STRING | --from FILE]` — bootstrap a managed file. Fails if sidecar exists.
  - `rhio apply <path> [--content STRING | --from FILE]` — reconcile. Idempotent.
  - `rhio status <path>` — drift report.
  - `rhio diff <path> [--content STRING | --from FILE]` — preview current block vs proposed content.
- Test coverage: 23 unit tests + 12 integration tests = **35 total**, all green.
- CI workflow: fmt + clippy + tests, runs on PRs against main and on push to main.
- Release workflow: tag-triggered cross-compile for darwin-arm64, darwin-x86_64, linux-x86_64, linux-arm64. Produces `.tar.gz` + `.sha256` per target, uploads to GitHub Releases. **Validated** with `v0.0.1-test`.
- Branch protection on main.

### Pending for v0.1.0 final

- [ ] Tag `v0.1.0` to publish first real release.
- [ ] Create `rhuffus/homebrew-rhio` tap with `Formula/rhio.rb` consuming the v0.1.0 release artifacts.
- [ ] Manual smoke test: `brew tap rhuffus/rhio && brew install rhio && rhio --version`.
- [ ] Archive `rhuffus/rhuffus-system-config-pack` with redirect in its README.

### Known limitations (will be addressed in later versions)

- Drift detection compares only the **hash** of the block content. Outside-block edits don't trigger drift in v0.1 (by design). v0.6 adds semantic drift.
- `diff` shows current vs proposed content as two listings, not a real unified diff. Will be improved.
- No automatic discovery of managed files — every command takes an explicit path.
- No per-host layering yet (v0.4).

## v0.2.0 — More text-based configs

- Parser support for INI (`gitconfig`) and `ssh-config` formats.
- Same model (block + sidecar) applied to:
  - `~/.gitconfig`
  - `~/.ssh/config`
- `rhio init/apply/status/diff` work uniformly across `.bashrc`, `.zshrc`, `.gitconfig`, `.ssh/config`.

## v0.3.0 — Structured files

- AST-level merge for JSON, YAML, TOML using tree-sitter parsers.
- Targets:
  - `~/.kube/config`
  - `~/.docker/config.json`
  - `~/.warp/settings.toml`
- The model differs from block markers: `rhio` injects / overrides specific fields and leaves the rest untouched. No block delimiter — that would break the document structure.
- Sidecar still applies, recording what fields rhio owns.

## v0.4.0 — Per-host layering

- Autodetect hostname via `hostname -s`.
- Source layout: `managed/base/<rel_path>` + `managed/hosts/<hostname>/<rel_path>`.
- During `apply`, the effective managed block is the concatenation of base + host overrides, with the host layer winning on per-key conflicts.
- Open question: should `rhio init` ask which layer to write to (base by default)?

## v0.5.0 — 1Password Environments integration

- Templates with `op://Vault/Item/field` references can be rendered into managed content.
- Local development: mount the rendered content as a real `.env` via 1Password's local-mount feature. Biometric prompt 1× per unlock cycle, all subsequent reads succeed without re-prompting.
- CI/CD: use a 1Password Service Account token; pipeline renders `<file>.tpl` → real file at deploy time. No biometric, no human in the loop.

## v0.6.0 — Interactive reconciliation

The headline feature.

When `rhio reconcile` (or `rhio apply` in v0.6 mode) runs against a managed file:

1. Detect stanzas outside the managed block.
2. For each, compare semantically (tree-sitter) against the proposed managed block content:
   - If equivalent (e.g., `FOO=bar` outside vs `export FOO=bar` inside) → silently remove from outside the block (dedup).
   - If hash matches an `ignored` entry in the sidecar → skip silently.
3. For stanzas with no equivalent in the block and no ignore record, prompt the user:
   - **promote** → add this stanza to the managed block content (accumulates in a pending batch).
   - **ignore** → record its hash in `<file>.rhio` `ignored`. Never prompted again.
   - **delete** → remove from the file.
4. After all files are reconciled, if any "promote" choices were made, create **one** commit + tag in the source repo containing all batched additions. Not one commit per file.

This is the build-vs-adopt justification: chezmoi/yadm don't do this and can't be retrofitted to do it cleanly.

## v1.0.0 — Polish & docs

- Comprehensive end-to-end tests across all supported file types and layered combinations.
- `CONTRIBUTING.md` with style guide, test conventions, release ritual.
- `docs/` site (mdBook or similar).
- Stable v1 API and CLI guarantees.

## Open architectural questions

These are not yet decided:

- **Registry of managed files**: should `rhio` maintain a `~/.config/rhio/managed.toml` registry, or always operate per-explicit-path? Trade-off: registry enables `rhio status` (no args) listing all files, but adds central state and a sync question.
- **Tool-edited dotfiles**: how to handle files mutated by their own tools (e.g., `git config --global` rewrites `.gitconfig`)? Should rhio detect & re-reconcile?
- **Per-machine secret partitioning**: how to distinguish "work credentials" from "personal" inside the hostname-layered model? Profiles on top of hostnames?
- **`rhio doctor`**: should there be a single command that audits every managed file at once and reports?
