mod fingerprinting {
    pub mod algorithm;
    pub mod communication;
    mod hanning;
    pub mod signature_format;
    mod user_agent;
}

mod core {
    pub mod http_thread;
}

use std::{
    env::args,
    fs::File,
    io::BufWriter,
    sync::{Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{core::http_thread::try_recognize_song, fingerprinting::algorithm::SignatureGenerator};

fn main() {
    let path = if let Some(path) = args().nth(1) {
        path
    } else {
        let host = cpal::default_host();
        println!("{:#?}", host.id());

        let device = host.default_output_device().expect("default_output_device");
        println!("{:#?}", device.name());

        let config = device
            .default_output_config()
            .expect("default_input_config");
        println!("{:#?}", config);

        let err_fn = move |err| {
            eprintln!("an error occurred on stream: {}", err);
        };

        let spec = wav_spec_from_config(&config);
        let writer = hound::WavWriter::create("file.wav", spec).unwrap();
        let writer = Arc::new(Mutex::new(Some(writer)));

        let stream = {
            let writer = writer.clone();

            match config.sample_format() {
                cpal::SampleFormat::F32 => device
                    .build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<f32, f32>(data, &writer),
                        err_fn,
                    )
                    .unwrap(),
                cpal::SampleFormat::I16 => device
                    .build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<i16, i16>(data, &writer),
                        err_fn,
                    )
                    .unwrap(),
                cpal::SampleFormat::U16 => device
                    .build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<u16, i16>(data, &writer),
                        err_fn,
                    )
                    .unwrap(),
            }
        };
        stream.play().unwrap();

        // Let recording go for roughly three seconds.
        std::thread::sleep(std::time::Duration::from_secs(3));
        drop(stream);

        writer.lock().unwrap().take().unwrap().finalize().unwrap();
        drop(writer);

        "file.wav".to_string()
    };

    println!("make_signature_from_file");
    let signature = SignatureGenerator::make_signature_from_file(&path).unwrap();

    println!("try_recognize_song");
    let result = try_recognize_song(signature).unwrap();

    println!("{}", result.track_key);
    println!("{}", result.album_name.unwrap_or_default());
    println!("{} by {}", result.song_name, result.artist_name);
}

type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: cpal::Sample,
    U: cpal::Sample + hound::Sample,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = cpal::Sample::from(&sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    match format {
        cpal::SampleFormat::U16 => hound::SampleFormat::Int,
        cpal::SampleFormat::I16 => hound::SampleFormat::Int,
        cpal::SampleFormat::F32 => hound::SampleFormat::Float,
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}
