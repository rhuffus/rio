use anyhow::Result;
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Bootstrap a managed file with the rio managed block.
#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Path to the file to bring under management.
    pub path: PathBuf,
}

pub fn run(_args: Args) -> Result<()> {
    anyhow::bail!("init: not yet implemented (v0.1 MVP work in progress)")
}
