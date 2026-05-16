use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use std::fs;
use std::path::PathBuf;

use crate::{block, sidecar};

/// Bootstrap a managed file with the rio managed block. Fails if the sidecar
/// already exists — use `rio apply` to reconcile an already-managed file.
#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Path to the file to bring under management.
    pub path: PathBuf,

    /// Inline content for the managed block.
    #[arg(long, conflicts_with = "from")]
    pub content: Option<String>,

    /// Read the managed block content from this file.
    #[arg(long, conflicts_with = "content")]
    pub from: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let sidecar_path = sidecar::path_for(&args.path);
    if sidecar_path.exists() {
        bail!(
            "sidecar already exists at {}; use `rio apply` to reconcile",
            sidecar_path.display()
        );
    }

    let content = crate::cli::resolve_content(args.content.as_deref(), args.from.as_deref())?;

    let existing = if args.path.exists() {
        fs::read_to_string(&args.path)
            .with_context(|| format!("reading {}", args.path.display()))?
    } else {
        String::new()
    };
    let updated = block::upsert(&existing, &content)?;
    fs::write(&args.path, &updated).with_context(|| format!("writing {}", args.path.display()))?;

    let sc = sidecar::Sidecar {
        managed_block_hash: Some(sidecar::hash_block(&content)),
        ..sidecar::Sidecar::default()
    };
    sc.save(&sidecar_path)?;

    println!(
        "rio: initialized {} ({} bytes managed)",
        args.path.display(),
        content.len()
    );
    Ok(())
}
