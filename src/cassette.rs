use std::{f32::consts::PI, iter, ops::Range};

use anyhow::{bail, ensure, Result};
use bitvec::{order::Msb0, vec::BitVec, view::BitView};

use crate::join_arrays;

/// Distance away from zero to consider a crossing.
/// This is used to reduce the impact of noise on the signal.
pub const CROSS_THRESHOLD: f32 = 0.1;

// The length of each pulse type in seconds.
pub const PULSE_ONE: Range<f32> = (15.0 / 44100.0)..(20.0 / 44100.0);
pub const PULSE_ZERO: Range<f32> = (35.0 / 44100.0)..(39.0 / 44100.0);
pub const PULSE_START: Range<f32> = (41.0 / 44100.0)..(46.0 / 44100.0);
pub const PULSE_END: f32 = 20000.0 / 44100.0;

/// The start sequence is 01111111.
const START_SEQUENCE: u8 = 0x7F;

#[derive(Debug)]
enum Pulse {
    Start,
    Zero,
    One,
}

pub struct Spec {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
}

pub fn decode(samples: &[i32], spec: Spec) -> Result<Vec<BitVec<u8, Msb0>>> {
    let max_value = ((1_u64 << spec.bits_per_sample/* - 1 */) - 1) as u32;
    let cross_threshold = (CROSS_THRESHOLD * max_value as f32) as i32;

    let mut intersections = Vec::new();
    let mut last = (0_i32, 0_usize);
    for (i, sample) in samples.iter().enumerate() {
        if i % spec.channels as usize != 0 {
            continue;
        }

        if sample.abs() > cross_threshold {
            if last.0.signum() != sample.signum() && last.0.signum() == -1 {
                intersections.push(i);
            }
            last = (*sample, i);
        }
    }

    let mut sections = Vec::new();
    let mut dat = Vec::new();
    for i in 0..intersections.len() - 1 {
        let diff = (intersections[i + 1] - intersections[i]) as f32 / spec.sample_rate as f32;
        if PULSE_ONE.contains(&diff) {
            dat.push(Pulse::One);
        } else if PULSE_ZERO.contains(&diff) {
            dat.push(Pulse::Zero);
        } else if PULSE_START.contains(&diff) {
            dat.push(Pulse::Start);
        } else if diff > PULSE_END {
            sections.push(dat);
            dat = Default::default();
        } else {
            bail!("Invalid pulse length: {diff}s");
        }
    }

    if !dat.is_empty() {
        sections.push(dat);
    }

    let mut raw_sections = Vec::new();
    let mut dat = BitVec::<u8, Msb0>::new();
    for section in sections.iter_mut() {
        let mut active = false;
        for pulse in section {
            match pulse {
                Pulse::Zero => dat.push(false),
                Pulse::One => dat.push(true),
                Pulse::Start if active => ensure!(dat.len() % 8 == 0, "Invalid start pulse"),
                Pulse::Start => dat.push(false),
            }

            if !active
                && dat.len() >= 8
                && dat[dat.len() - 8..] == START_SEQUENCE.view_bits::<Msb0>()
            {
                active = true;
                dat.clear();
            }
        }

        ensure!(active, "Didn't find start sequence");
        raw_sections.push(dat);
        dat = Default::default();
    }

    Ok(raw_sections)
}

// 0-.7
pub fn encode(data: &[&[u8]], spec: &Spec) -> Result<Vec<i32>> {
    let header_data = join_arrays!([0x55; 255], [0x7F]);
    let header = encode_segment(&header_data, spec)?;

    let mut out = Vec::new();
    for (i, dat) in data.iter().enumerate() {
        out.extend(header.iter());
        out.extend(encode_segment(dat, spec)?);
        if i != data.len() - 1 {
            out.extend(iter::repeat(0).take((spec.sample_rate as f32 * 0.75) as usize));
        }
    }

    Ok(out)
}

fn encode_segment(data: &[u8], spec: &Spec) -> Result<Vec<i32>> {
    let max_value = ((1_u64 << (spec.bits_per_sample - 1)) - 1) as f32;

    // 0 => 2680 Hz, 1 => 1320 Hz
    let freq = |x: f32| (spec.sample_rate as f32 / x).round() as u32;
    let [samples_0, samples_1] = [freq(1320.0), freq(2680.0)];

    let data = data.view_bits::<Msb0>();
    let mut out = Vec::new();

    for bit in data {
        let samples = if *bit { samples_1 } else { samples_0 };
        let freq = if *bit { 2680.0 } else { 1320.0 };
        for i in 0..samples {
            let val = (i as f32 * freq * 2.0 * PI / spec.sample_rate as f32).sin();
            out.push((val * max_value) as i32);
        }
    }

    Ok(out)
}

impl From<hound::WavSpec> for Spec {
    fn from(spec: hound::WavSpec) -> Self {
        Self {
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            bits_per_sample: spec.bits_per_sample,
        }
    }
}

impl From<cpal::SupportedStreamConfig> for Spec {
    fn from(spec: cpal::SupportedStreamConfig) -> Self {
        Self {
            sample_rate: spec.sample_rate().0,
            channels: spec.channels(),
            bits_per_sample: spec.sample_format().sample_size() as u16,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_encode_segment() {
        use super::*;

        let spec = Spec {
            sample_rate: 44100,
            channels: 1,
            bits_per_sample: 16,
        };

        let data: &[&[u8]] = &[b"Hello, world!"];
        let encoded = encode(data, &spec).unwrap();

        let mut wav = hound::WavWriter::create(
            "output-test.wav",
            hound::WavSpec {
                channels: 1,
                sample_rate: 44100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();

        for sample in encoded {
            wav.write_sample(sample).unwrap();
        }

        wav.finalize().unwrap();
    }
}
