use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    let device = host.default_input_device().expect("No input device found");

    println!("Using device: {}", device.description()?);

    let supported_config = device.default_input_config()?;
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    let spec = hound::WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = hound::WavWriter::create("recorded.wav", spec)?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let is_recording = Arc::new(AtomicBool::new(true));

    {
        let is_recording = is_recording.clone();

        ctrlc::set_handler(move || {
            println!("\nStopping recording...");
            is_recording.store(false, Ordering::SeqCst);
        })?;
    }

    let writer_clone = writer.clone();

    let err_fn = |err| {
        eprintln!("Stream error: {}", err);
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, writer_clone, err_fn)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, writer_clone, err_fn)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, writer_clone, err_fn)?,
        _ => panic!("Unsupported sample format"),
    };

    stream.play()?;

    println!("Recording... Press Ctrl+C to stop.");

    while is_recording.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    drop(stream);

    writer.lock().unwrap().take().unwrap().finalize()?;

    println!("Saved recording to recorded.wav");

    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    writer: Arc<Mutex<Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample + cpal::SizedSample,
    i16: cpal::FromSample<T>,
{
    let channels = config.channels as usize;

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut writer = writer.lock().unwrap();

            if let Some(writer) = writer.as_mut() {
                for frame in data.chunks(channels) {
                    for sample in frame {
                        let sample: i16 = sample.to_sample::<i16>();
                        let _ = writer.write_sample(sample);
                    }
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}
