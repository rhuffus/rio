use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::fs;
use std::path::PathBuf;

use crate::{block, sidecar};

/// Report drift between a managed file's block and its sidecar.
#[derive(ClapArgs, Debug)]
pub struct Args {
    pub path: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    let text = fs::read_to_string(&args.path)
        .with_context(|| format!("reading {}", args.path.display()))?;
    let block = block::find(&text)?;
    let sidecar_path = sidecar::path_for(&args.path);
    let sc = if sidecar_path.exists() {
        Some(sidecar::Sidecar::load(&sidecar_path)?)
    } else {
        None
    };

    match (block, sc) {
        (None, None) => {
            println!("{}: not managed", args.path.display());
        }
        (None, Some(_)) => {
            println!(
                "{}: sidecar exists but block is missing from file",
                args.path.display()
            );
        }
        (Some(b), None) => {
            println!(
                "{}: block present (lines {}..{}) but no sidecar — run `rio init` or `rio apply`",
                args.path.display(),
                b.start_line + 1,
                b.end_line + 1
            );
        }
        (Some(b), Some(sc)) => {
            let current_hash = sidecar::hash_block(&b.content);
            match sc.managed_block_hash.as_deref() {
                Some(stored) if stored == current_hash => {
                    println!("{}: clean", args.path.display());
                }
                Some(stored) => {
                    println!(
                        "{}: DRIFT (block changed since last apply)\n  block hash:   {current_hash}\n  sidecar hash: {stored}",
                        args.path.display()
                    );
                }
                None => {
                    println!(
                        "{}: block present but sidecar has no recorded hash; run `rio apply`",
                        args.path.display()
                    );
                }
            }
        }
    }
    Ok(())
}
