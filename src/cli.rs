use anyhow::Result;
use clap::{Parser, Subcommand};

pub mod apply;
pub mod diff;
pub mod init;
pub mod status;

#[derive(Parser, Debug)]
#[command(name = "rio", version, about = "Block-based reconciliation dotfile manager.")]
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
