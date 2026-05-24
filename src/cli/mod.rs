use clap::{Parser, Subcommand};

mod devices;
mod record;

#[derive(Parser)]
struct Client {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "List all audio devices")]
    Devices(devices::DevicesCmd),

    #[command(about = "Record audio to a file (format from extension)")]
    Record(record::RecordCmd),
}

impl Commands {
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            Commands::Devices(cmd) => cmd.run(),
            Commands::Record(cmd) => cmd.run(),
        }
    }
}

trait Cmd {
    fn run(&self) -> anyhow::Result<()>;
}

pub fn run() -> anyhow::Result<()> {
    Client::parse().command.run()
}
