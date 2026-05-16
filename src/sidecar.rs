//! `.rio` sidecar file format.
//!
//! Each managed file has a sibling `<file>.rio` that persists plugin state:
//! current block content hash, ignored-stanza hashes, host overrides, etc.
//! Format: TOML (versioned in git for portability across machines).
