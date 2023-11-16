use std::fs;

use bitvec::{array::BitArray, bitarr, order::Msb0, vec::BitVec};
use hound::{self, WavReader};

const CROSS_THRESHOLD: f32 = 0.1;
const INT_CROSS_THRESHOLD: i32 = (CROSS_THRESHOLD * i16::MAX as f32) as i32;

#[derive(Debug)]
enum Pulse {
    Start,
    Zero,
    One,
}

fn main() {
    let mut reader = WavReader::open("test.wav").unwrap();
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
    // TODO: These are tied to the sample rate...
    for i in 0..intersections.len() - 1 {
        let diff = intersections[i + 1] - intersections[i];
        if (15..20).contains(&diff) {
            dat.push(Pulse::One);
        } else if (35..39).contains(&diff) {
            dat.push(Pulse::Zero);
        } else if (41..46).contains(&diff) {
            dat.push(Pulse::Start);
        } else if diff > 30000 {
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

            // If data equals [0, 1, 1, 1, 1, 1, 1, 1] then active = true
            const START_SEQUENCE: BitArray<[u8; 1], Msb0> =
                bitarr![const u8, Msb0; 0, 1, 1, 1, 1, 1, 1, 1];
            if !active && dat.len() >= 8 && &dat[dat.len() - 8..] == &START_SEQUENCE {
                active = true;
                dat.clear();
            }
        }

        assert!(active, "Didn't find start sequence");
        raw_sections.push(dat);
        dat = Default::default();
    }

    for (i, section) in raw_sections.iter().enumerate() {
        fs::write(format!("out/section-{i}.bin"), section.as_raw_slice()).unwrap();
    }
}
