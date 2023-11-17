use std::{fs::File, io::BufReader, ops::Range};

use bitvec::{order::Msb0, vec::BitVec, view::BitView};
use hound::WavReader;

/// Distance away from zero to consider a crossing.
/// This is used to reduce the impact of noise on the signal.
const CROSS_THRESHOLD: f32 = 0.1;

// The length of each pulse type in seconds.
const PULSE_ONE: Range<f32> = (15.0 / 44100.0)..(20.0 / 44100.0);
const PULSE_ZERO: Range<f32> = (35.0 / 44100.0)..(39.0 / 44100.0);
const PULSE_START: Range<f32> = (41.0 / 44100.0)..(46.0 / 44100.0);
const PULSE_END: f32 = 20000.0 / 44100.0;

/// The start sequence is 01111111.
const START_SEQUENCE: u8 = 0x7F;
const INT_CROSS_THRESHOLD: i32 = (CROSS_THRESHOLD * i16::MAX as f32) as i32;

#[derive(Debug)]
enum Pulse {
    Start,
    Zero,
    One,
}

pub fn decode(reader: &mut WavReader<BufReader<File>>) -> Vec<BitVec<u8, Msb0>> {
    let spec = reader.spec();

    let mut intersections = Vec::new();
    let mut last = (0_i32, 0_usize);
    for (i, sample) in reader.samples::<i32>().map(|x| x.unwrap()).enumerate() {
        if i % spec.channels as usize != 0 {
            continue;
        }

        if sample.abs() > INT_CROSS_THRESHOLD {
            if last.0.signum() != sample.signum() && last.0.signum() == -1 {
                intersections.push(i);
            }
            last = (sample, i);
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
            panic!("Invalid pulse length: {}", diff);
        }
    }

    if !dat.is_empty() {
        sections.push(dat);
    }

    println!("[I] Found {} sections", sections.len());

    let mut raw_sections = Vec::new();
    let mut dat = BitVec::<u8, Msb0>::new();
    for section in sections.iter_mut() {
        let mut active = false;
        for pulse in section {
            match pulse {
                Pulse::Zero => dat.push(false),
                Pulse::One => dat.push(true),
                Pulse::Start if active => assert!(dat.len() % 8 == 0),
                Pulse::Start => dat.push(false),
            }

            if !active
                && dat.len() >= 8
                && &dat[dat.len() - 8..] == START_SEQUENCE.view_bits::<Msb0>()
            {
                active = true;
                dat.clear();
            }
        }

        assert!(active, "Didn't find start sequence");
        raw_sections.push(dat);
        dat = Default::default();
    }

    raw_sections
}
