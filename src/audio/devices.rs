use cpal::traits::{DeviceTrait, HostTrait};

pub fn list_devices() -> anyhow::Result<()> {
    let host = cpal::default_host();

    for device in host.devices()? {
        println!("{}", device.description()?);
    }

    Ok(())
}
