use clap::Args;

use crate::audio;

#[derive(Args)]
pub struct DevicesCmd;

impl super::Cmd for DevicesCmd {
    fn run(&self) -> anyhow::Result<()> {
        audio::devices::list_devices()?;
        Ok(())
    }
}
