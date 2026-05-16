use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

pub mod apply;
pub mod diff;
pub mod init;
pub mod status;

#[derive(Parser, Debug)]
#[command(
    name = "rio",
    version,
    about = "Block-based reconciliation dotfile manager."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init(init::Args),
    Status(status::Args),
    Apply(apply::Args),
    Diff(diff::Args),
}

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::Init(args) => init::run(args),
            Command::Status(args) => status::run(args),
            Command::Apply(args) => apply::run(args),
            Command::Diff(args) => diff::run(args),
        }
    }
}

/// Shared helper for subcommands that accept block content via `--content` or
/// `--from`. When both are absent the content is empty.
pub fn resolve_content(content: Option<&str>, from: Option<&Path>) -> Result<String> {
    if let Some(c) = content {
        let mut s = c.to_string();
        if !s.ends_with('\n') {
            s.push('\n');
        }
        return Ok(s);
    }
    if let Some(p) = from {
        return std::fs::read_to_string(p)
            .with_context(|| format!("reading content from {}", p.display()));
    }
    Ok(String::new())
}
