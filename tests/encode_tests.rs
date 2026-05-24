fn generate_sine_wave(samples: usize, channels: u16, sample_rate: u32) -> Vec<i16> {
    let mut data = Vec::with_capacity(samples * channels as usize);
    let freq = 440.0;
    let amplitude = 16000i16;

    for i in 0..samples {
        let t = i as f64 / sample_rate as f64;
        let value = (amplitude as f64 * (2.0 * std::f64::consts::PI * freq * t).sin()) as i16;
        for _ in 0..channels {
            data.push(value);
        }
    }
    data
}

#[test]
fn test_encode_wav() {
    let dir = std::env::temp_dir();
    let path = dir.join("test_encode.wav");
    let path_str = path.to_str().unwrap();

    let samples = generate_sine_wave(4410, 2, 44100);
    vengine::audio::record::encode_via_ffmpeg(path_str, &samples, 2, 44100).unwrap();

    assert!(path.exists());
    let data = std::fs::read(path_str).unwrap();
    assert!(data.len() > 44, "WAV file too small");
    assert_eq!(&data[0..4], b"RIFF", "RIFF header missing");
    assert_eq!(&data[8..12], b"WAVE", "WAVE header missing");

    std::fs::remove_file(path_str).ok();
}

#[test]
fn test_encode_flac() {
    let dir = std::env::temp_dir();
    let path = dir.join("test_encode.flac");
    let path_str = path.to_str().unwrap();

    let samples = generate_sine_wave(4410, 2, 44100);
    vengine::audio::record::encode_via_ffmpeg(path_str, &samples, 2, 44100).unwrap();

    assert!(path.exists());
    let data = std::fs::read(path_str).unwrap();
    assert!(data.len() > 50, "FLAC file too small");
    assert_eq!(&data[0..4], b"fLaC", "FLAC magic bytes missing");

    std::fs::remove_file(path_str).ok();
}

#[test]
fn test_encode_ogg() {
    let dir = std::env::temp_dir();
    let path = dir.join("test_encode.ogg");
    let path_str = path.to_str().unwrap();

    let samples = generate_sine_wave(4410, 2, 44100);
    vengine::audio::record::encode_via_ffmpeg(path_str, &samples, 2, 44100).unwrap();

    assert!(path.exists());
    let data = std::fs::read(path_str).unwrap();
    assert!(data.len() > 50, "OGG file too small");
    assert_eq!(&data[0..4], b"OggS", "OGG magic bytes missing");

    std::fs::remove_file(path_str).ok();
}

#[test]
fn test_encode_mp3() {
    let dir = std::env::temp_dir();
    let path = dir.join("test_encode.mp3");
    let path_str = path.to_str().unwrap();

    let samples = generate_sine_wave(4410, 2, 44100);
    vengine::audio::record::encode_via_ffmpeg(path_str, &samples, 2, 44100).unwrap();

    assert!(path.exists());
    let data = std::fs::read(path_str).unwrap();
    assert!(data.len() > 100, "MP3 file too small");
    let has_sync = data[0] == 0xFF && (data[1] & 0xE0) == 0xE0;
    let has_id3 = data.len() >= 3 && &data[0..3] == b"ID3";
    assert!(has_sync || has_id3, "MP3 sync bytes or ID3 header missing");

    std::fs::remove_file(path_str).ok();
}

#[test]
fn test_encode_mono_wav() {
    let dir = std::env::temp_dir();
    let path = dir.join("test_mono.wav");
    let path_str = path.to_str().unwrap();

    let samples = generate_sine_wave(4410, 1, 44100);
    vengine::audio::record::encode_via_ffmpeg(path_str, &samples, 1, 44100).unwrap();

    assert!(path.exists());
    let data = std::fs::read(path_str).unwrap();
    assert_eq!(&data[0..4], b"RIFF", "RIFF header missing");

    std::fs::remove_file(path_str).ok();
}

#[test]
fn test_ffmpeg_invalid_path_error() {
    let dir = std::env::temp_dir();
    let path = dir.join("\0invalid\0.wav");
    let path_str = path.to_str().unwrap();

    let samples = vec![0i16; 100];
    let result = vengine::audio::record::encode_via_ffmpeg(path_str, &samples, 1, 44100);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("ffmpeg"),
        "Error should mention ffmpeg: {err}"
    );
}
