use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::fs;
use std::path::PathBuf;

use crate::block;

/// Show what `apply` would change in the managed block.
///
/// Prints the current block content and the proposed content side by side.
/// A semantic diff (tree-sitter-aware) lands in a later version.
#[derive(ClapArgs, Debug)]
pub struct Args {
    pub path: PathBuf,

    #[arg(long, conflicts_with = "from")]
    pub content: Option<String>,

    #[arg(long, conflicts_with = "content")]
    pub from: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let new_content = crate::cli::resolve_content(args.content.as_deref(), args.from.as_deref())?;

    let text = if args.path.exists() {
        fs::read_to_string(&args.path)
            .with_context(|| format!("reading {}", args.path.display()))?
    } else {
        String::new()
    };
    let current = match block::find(&text)? {
        Some(b) => b.content,
        None => String::new(),
    };

    if current == new_content {
        println!("{}: no changes", args.path.display());
        return Ok(());
    }

    println!("=== current block ({}) ===", args.path.display());
    if current.is_empty() {
        println!("(empty)");
    } else {
        print!("{current}");
    }
    println!("=== proposed block ===");
    if new_content.is_empty() {
        println!("(empty)");
    } else {
        print!("{new_content}");
    }
    Ok(())
}
