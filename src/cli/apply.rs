use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::fs;
use std::path::PathBuf;

use crate::{block, sidecar};

/// Reconcile a managed file: replace the block with the provided content and
/// update the sidecar's hash. Idempotent.
#[derive(ClapArgs, Debug)]
pub struct Args {
    pub path: PathBuf,

    #[arg(long, conflicts_with = "from")]
    pub content: Option<String>,

    #[arg(long, conflicts_with = "content")]
    pub from: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let content = crate::cli::resolve_content(args.content.as_deref(), args.from.as_deref())?;

    let existing = if args.path.exists() {
        fs::read_to_string(&args.path)
            .with_context(|| format!("reading {}", args.path.display()))?
    } else {
        String::new()
    };
    let updated = block::upsert(&existing, &content)?;
    fs::write(&args.path, &updated)
        .with_context(|| format!("writing {}", args.path.display()))?;

    let sidecar_path = sidecar::path_for(&args.path);
    let mut sc = if sidecar_path.exists() {
        sidecar::Sidecar::load(&sidecar_path)?
    } else {
        sidecar::Sidecar::default()
    };
    sc.managed_block_hash = Some(sidecar::hash_block(&content));
    sc.save(&sidecar_path)?;

    println!("rio: applied {} (block hash updated)", args.path.display());
    Ok(())
}
