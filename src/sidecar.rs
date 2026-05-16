//! `.rio` sidecar file format.
//!
//! Each managed file at `<path>` has a sibling sidecar at `<path>.rio` that
//! persists plugin state: the hash of the current managed block, decisions the
//! user made about non-managed stanzas (`ignored`), etc. The sidecar is TOML
//! and intended to be versioned in git for portability across machines.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sidecar {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub managed_block_hash: Option<String>,
    #[serde(default)]
    pub ignored: Vec<String>,
}

impl Default for Sidecar {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            managed_block_hash: None,
            ignored: Vec::new(),
        }
    }
}

impl Sidecar {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("reading sidecar at {}", path.display()))?;
        let parsed: Self = toml::from_str(&raw)
            .with_context(|| format!("parsing sidecar at {}", path.display()))?;
        Ok(parsed)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let raw = toml::to_string_pretty(self).context("serializing sidecar")?;
        fs::write(path, raw)
            .with_context(|| format!("writing sidecar at {}", path.display()))?;
        Ok(())
    }
}

/// For a managed file at `<file>`, the sidecar lives at `<file>.rio`.
pub fn path_for(file: &Path) -> PathBuf {
    let mut s = file.as_os_str().to_os_string();
    s.push(".rio");
    PathBuf::from(s)
}

/// Canonical hash of a managed block's content. Used for drift detection.
pub fn hash_block(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_sidecar_uses_current_schema() {
        let s = Sidecar::default();
        assert_eq!(s.version, SCHEMA_VERSION);
        assert!(s.managed_block_hash.is_none());
        assert!(s.ignored.is_empty());
    }

    #[test]
    fn path_for_appends_rio_extension() {
        let p = path_for(Path::new("/home/u/.zshrc"));
        assert_eq!(p, PathBuf::from("/home/u/.zshrc.rio"));
    }

    #[test]
    fn path_for_works_with_no_extension() {
        let p = path_for(Path::new("/etc/hosts"));
        assert_eq!(p, PathBuf::from("/etc/hosts.rio"));
    }

    #[test]
    fn hash_is_stable_and_distinct() {
        assert_eq!(hash_block("foo"), hash_block("foo"));
        assert_ne!(hash_block("foo"), hash_block("bar"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rio");
        let original = Sidecar {
            version: SCHEMA_VERSION,
            managed_block_hash: Some("abc123".to_string()),
            ignored: vec!["hash1".to_string(), "hash2".to_string()],
        };
        original.save(&path).unwrap();
        let loaded = Sidecar::load(&path).unwrap();
        assert_eq!(original, loaded);
    }

    #[test]
    fn load_returns_error_for_invalid_toml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.rio");
        fs::write(&path, "not [ valid toml").unwrap();
        assert!(Sidecar::load(&path).is_err());
    }
}
