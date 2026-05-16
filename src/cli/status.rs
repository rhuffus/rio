use anyhow::Result;
use clap::Args as ClapArgs;

/// Show drift between managed blocks and the current file state.
#[derive(ClapArgs, Debug)]
pub struct Args {}

pub fn run(_args: Args) -> Result<()> {
    anyhow::bail!("status: not yet implemented (v0.1 MVP work in progress)")
}
