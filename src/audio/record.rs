use std::{
    io::Write,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn record_to_file(path: &str) -> anyhow::Result<()> {
    let host = cpal::default_host();

    let device = host.default_input_device().expect("No input device found");

    println!("Using device: {}", device.description()?);

    let supported_config = device.default_input_config()?;
    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.into();

    println!("Recording config: {:#?}", config);

    let samples: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
    let is_recording = Arc::new(AtomicBool::new(true));

    {
        let is_recording = is_recording.clone();

        ctrlc::set_handler(move || {
            println!("\nStopping recording...");
            is_recording.store(false, Ordering::SeqCst);
        })?;
    }

    let samples_clone = samples.clone();

    let err_fn = |err| {
        eprintln!("Stream error: {}", err);
    };

    let stream = match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config, samples_clone, err_fn)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config, samples_clone, err_fn)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config, samples_clone, err_fn)?,
        _ => panic!("Unsupported sample format"),
    };

    stream.play()?;

    println!("Recording... Press Ctrl+C to stop.");

    while is_recording.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    drop(stream);

    let recorded = samples.lock().unwrap().clone();
    let channels = config.channels;
    let sample_rate = config.sample_rate;

    encode_via_ffmpeg(path, &recorded, channels, sample_rate)?;

    println!("Saved recording to {}", path);

    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples: Arc<Mutex<Vec<i16>>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: cpal::Sample + cpal::SizedSample,
    i16: cpal::FromSample<T>,
{
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let mut samples = samples.lock().unwrap();

            for sample in data {
                let sample: i16 = sample.to_sample::<i16>();
                samples.push(sample);
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

pub fn encode_via_ffmpeg(path: &str, samples: &[i16], channels: u16, sample_rate: u32) -> anyhow::Result<()> {
    let mut child = Command::new("C:\\Program Files\\FFMPEG\\ffmpeg-8.1.1-full_build\\bin\\ffmpeg.exe")
        .args(["-y", "-f", "s16le", "-ar", &sample_rate.to_string()])
        .args(["-ac", &channels.to_string(), "-i", "pipe:0"])
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to launch ffmpeg: {e}"))?;

    let mut stdin = child.stdin.take().unwrap();
    let bytes = unsafe {
        std::slice::from_raw_parts(samples.as_ptr() as *const u8, samples.len() * 2)
    };
    stdin.write_all(bytes)?;
    drop(stdin);

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg failed: {stderr}");
    }

    Ok(())
}
