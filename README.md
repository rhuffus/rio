# rhio

> **R**huffus**IO** — a block-based reconciliation dotfile manager.

`rhio` keeps your dotfiles synchronized across machines without overwriting them. It manages a delimited block inside each config file you bring under its control, leaving the rest of the file to you.

## What makes it different

Unlike traditional dotfile managers (chezmoi, yadm, GNU Stow), `rhio`:

- **Co-resides with your edits.** Only the marked block is owned by `rhio`. Everything else in the file stays yours.
- **Understands structure.** Uses tree-sitter to compare configurations semantically (`FOO=bar` and `export FOO=bar` are the same thing).
- **Reconciles drift interactively** *(v0.6).* Detects manual edits outside the block and offers to promote, ignore, or remove them.
- **Tracks decisions** in a `.rhio` sidecar next to each managed file — versioned in git, portable across machines.

## Status

**v0.1.0 — alpha.** The MVP ships managed-block handling for shell files (`.zshrc`, `.bashrc`) with tree-sitter-bash parsing and the `.rhio` sidecar format. See [docs/ROADMAP.md](docs/ROADMAP.md) for what each upcoming version adds.

## Install

```sh
# Via Homebrew tap (recommended, once v0.1.0 ships)
brew tap rhuffus/rhio
brew install rhio

# Or build from source
git clone https://github.com/rhuffus/rhio && cd rhio && cargo install --path .
```

## Usage

```sh
# Bootstrap a managed file with content for the rhio block
rhio init ~/.zshrc --from ./my-zsh-config.sh

# Report drift between the file's block and the sidecar's recorded hash
rhio status ~/.zshrc

# Preview what apply would change
rhio diff ~/.zshrc --from ./my-zsh-config.sh

# Reconcile: overwrite the block, update the sidecar hash
rhio apply ~/.zshrc --from ./my-zsh-config.sh
```

All commands accept `--content <STRING>` for inline content or `--from <FILE>` to read content from a file.

## How it works

A `rhio`-managed file looks like this:

```sh
# Your personal stuff above the block — rhio never touches this.
export EDITOR=nvim
alias gst='git status'

# >>> RhuffusIO Managed Block >>>
# Owned by rhio. Edits inside are subject to reconciliation.
export PATH="$HOME/.cargo/bin:$PATH"
alias k=kubectl
# <<< RhuffusIO Managed Block <<<

# Your personal stuff below the block — also untouched.
[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh
```

The sidecar at `~/.zshrc.rhio` records the hash of the managed block plus any "ignored" decisions:

```toml
version = 1
managed_block_hash = "a1b2c3..."
ignored = []
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the design rationale, the build-vs-adopt analysis against `chezmoi` and `yadm`, and the locked decisions.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — design model, locked decisions, comparison to existing dotfile managers
- [docs/ROADMAP.md](docs/ROADMAP.md) — detailed roadmap from v0.1 through v1.0
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — building, testing, branch model, release process

## License

Apache-2.0. See [LICENSE](./LICENSE).
