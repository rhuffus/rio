# rio

> **R**huffus**IO** — a block-based reconciliation dotfile manager.

`rio` keeps your dotfiles synchronized across machines without overwriting them. It manages a delimited block inside each config file you bring under its control, leaving the rest of the file to you.

Unlike traditional dotfile managers, `rio`:

- **Co-resides with your edits**: only the marked block is owned by `rio`. Everything else stays yours.
- **Reconciles drift**: detects manual edits outside the block and offers to promote, ignore, or remove them.
- **Understands structure**: uses tree-sitter parsers to compare configurations semantically, not just textually.
- **Tracks decisions**: a `.rio` sidecar next to each managed file persists what you have chosen to ignore.

## Status

`v0.1.0` — under active development. Not yet published.

### Roadmap

| Version | Scope |
|---------|-------|
| v0.1 (MVP) | Shell files (`.zshrc`, `.bashrc`), block markers, tree-sitter-bash, `.rio` sidecar, Homebrew tap |
| v0.2 | `.gitconfig`, `.ssh/config` (INI / ssh-config parsers) |
| v0.3 | Structured files (`.kube/config`, `.docker/config.json`, `.warp/settings.toml`) via AST merge |
| v0.4 | Per-host layering (`hostname -s` autodetected) |
| v0.5 | 1Password Environments integration (`op://` references, `op inject` for CI/CD) |
| v0.6 | Full interactive promote / ignore / delete with batched auto commit + tag |
| v1.0 | Polish & docs |

## Install

Not yet published. Once v0.1.0 ships:

```sh
brew tap rhuffus/rio
brew install rio
```

## Usage

```sh
rio init ~/.zshrc      # bootstrap a managed file
rio status             # show drift across all managed files
rio diff               # preview what apply would change
rio apply              # reconcile: write blocks, update sidecars
```

## License

Apache-2.0. See [LICENSE](./LICENSE).
