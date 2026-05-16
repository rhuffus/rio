use anyhow::Result;
use clap::Args as ClapArgs;

/// Show what `apply` would change without writing anything.
#[derive(ClapArgs, Debug)]
pub struct Args {}

pub fn run(_args: Args) -> Result<()> {
    anyhow::bail!("diff: not yet implemented (v0.1 MVP work in progress)")
}
