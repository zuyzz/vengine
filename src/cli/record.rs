use clap::Args;

use crate::audio;

#[derive(Args)]
pub struct RecordCmd {
    #[arg(short, long)]
    pub output: Option<String>,
}

impl super::Cmd for RecordCmd {
    fn run(&self) -> anyhow::Result<()> {
        let output = self.output.as_deref().unwrap_or("output.wav");
        audio::record::record_to_file(output)?;
        Ok(())
    }
}
