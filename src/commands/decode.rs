use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Barrier,
    },
};

use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    StreamInstant,
};
use hound::WavSpec;
use parking_lot::Mutex;

use crate::{
    args::{self, Decode},
    cassette::{CROSS_THRESHOLD, PULSE_END},
    formats::{raw, text},
    misc::{audio_dev, plural},
};

pub fn decode(args: Decode) -> Result<()> {
    if args.input_audio {
        decode_audio(args)
    } else {
        decode_file(args)
    }
}

fn decode_file(args: Decode) -> Result<()> {
    println!(
        "[*] Decoding `{}` to `{}` ({})",
        args.input,
        args.output.to_string_lossy(),
        args.format
    );

    let input = Path::new(&args.input);
    let reader = hound::WavReader::open(&input)?;
    println!(
        " ├─ {} channel{}, {} Hz, {} bit{}",
        reader.spec().channels,
        plural(reader.spec().channels),
        reader.spec().sample_rate,
        reader.spec().bits_per_sample,
        plural(reader.spec().bits_per_sample)
    );

    let spec = reader.spec().into();
    let samples = reader
        .into_samples()
        .collect::<Result<Vec<i32>, hound::Error>>()?;

    (match args.format {
        args::Format::Raw => raw::decode,
        args::Format::Text => text::decode,
    })(&samples, spec, args)?;

    println!("[*] Done!");
    Ok(())
}

fn decode_audio(args: Decode) -> Result<()> {
    let host = cpal::default_host();
    let device = audio_dev(host.input_devices()?, &args.input)?;
    println!("[*] Using audio device `{}`", device.name()?);
    let mut config_range = device.supported_input_configs()?;
    let config = config_range
        .next()
        .context("No configs")?
        .with_max_sample_rate();
    let sample_rate = config.sample_rate().0;
    let spec = config.clone().into();

    struct State {
        samples: Mutex<Vec<i32>>,
        barrier: Barrier,
    }

    let state = Arc::new(State {
        samples: Mutex::new(Vec::new()),
        barrier: Barrier::new(2),
    });

    let mut last_cross = None;
    let stream_state = state.clone();
    let stream = device.build_input_stream(
        &config.clone().into(),
        move |data: &[f32], info: &cpal::InputCallbackInfo| {
            let mut samples = stream_state.samples.lock();
            for sample in data {
                samples.push((sample * i32::MAX as f32) as i32);

                if let Some(last_cross) = last_cross {
                    if info
                        .timestamp()
                        .capture
                        .duration_since(&last_cross)
                        .unwrap()
                        .as_secs_f32()
                        > 3.0
                    {
                        println!("[*] Stopping capture");
                        stream_state.barrier.wait();
                        return;
                    }
                }

                if sample.abs() > CROSS_THRESHOLD {
                    if last_cross.is_none() {
                        println!("[*] Starting capture");
                    }

                    last_cross = Some(info.timestamp().capture);
                }

                if last_cross.is_none() {
                    while samples.len() > sample_rate as usize {
                        samples.remove(0);
                    }
                }
            }
        },
        move |err| eprintln!("Error: {}", err),
        None,
    )?;

    println!("[*] Waiting for audio input on `{}`", device.name()?);
    stream.play()?;

    state.barrier.wait();
    stream.pause()?;
    drop(stream);

    let mut wav_writer = hound::WavWriter::create(
        &"debug.wav",
        WavSpec {
            channels: config.clone().channels(),
            sample_rate: config.sample_rate().0,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Int,
        },
    )?;
    for sample in state.samples.lock().iter() {
        wav_writer.write_sample(*sample)?;
    }
    wav_writer.finalize()?;

    let samples = state.samples.lock();
    (match args.format {
        args::Format::Raw => raw::decode,
        args::Format::Text => text::decode,
    })(&samples, spec, args)?;

    Ok(())
}
