use anyhow::Result;
use clap::Parser;
use rhio::cli::Cli;

fn main() -> Result<()> {
    Cli::parse().run()
}
