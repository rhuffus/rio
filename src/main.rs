use anyhow::Result;
use clap::Parser;
use rio::cli::Cli;

fn main() -> Result<()> {
    Cli::parse().run()
}
