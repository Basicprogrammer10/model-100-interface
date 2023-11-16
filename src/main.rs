use std::fs::{self, File};

use bitvec::{order::Lsb0, slice::BitSlice, vec::BitVec};
use hound::{self, WavReader, WavSpec, WavWriter};

const CROSS_THRESHOLD: f32 = 0.1;
const INT_CROSS_THRESHOLD: i32 = (CROSS_THRESHOLD * i16::MAX as f32) as i32;

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
    let mut dat = BitVec::<u8, Lsb0>::new();
    // TODO: These are tied to the sample rate...
    for i in 0..intersections.len() - 1 {
        let diff = intersections[i + 1] - intersections[i];
        if (15..20).contains(&diff) {
            dat.push(true);
        } else if (35..39).contains(&diff) {
            dat.push(false);
        } else if (41..46).contains(&diff) {
            // Start pulse should be on index where % 9 == 0
            if dat.len() % 8 != 0 {
                println!("Invalid start pulse index: {}", dat.len());
                // dat.clear();
            }
        } else if diff > 30000 {
            sections.push(dat);
            dat = Default::default();
        } else {
            panic!("Invalid pulse length: {}", diff);
        }
    }

    let mut new_sections = Vec::new();
    for section in sections {
        let mut start = 30;
        // Find leading 0x55 bytes
        for (i, byte) in section.windows(8).enumerate() {
            if byte_eq(byte, &[false, true, false, true, false, true, false, true]) {
                start -= 1;
            } else if start < 30 {
                // new_sections.push(section[i..].to_bitvec());
                new_sections.push(section.to_bitvec());
                break;
            }
        }
    }

    let spec = WavSpec {
        channels: 1,
        bits_per_sample: 16,
        ..spec
    };
    let mut debug = WavWriter::new(File::create("debug/out.wav").unwrap(), spec).unwrap();
    let mut ones = WavWriter::new(File::create("debug/ones.wav").unwrap(), spec).unwrap();
    let mut zero = WavWriter::new(File::create("debug/zero.wav").unwrap(), spec).unwrap();
    let mut start = WavWriter::new(File::create("debug/start.wav").unwrap(), spec).unwrap();

    let mut last = 0;
    for i in intersections {
        let gap = i - last - 1;
        for _ in 0..gap {
            debug.write_sample(0).unwrap();
            ones.write_sample(0).unwrap();
            zero.write_sample(0).unwrap();
            start.write_sample(0).unwrap();
        }
        last = i;
        debug.write_sample(i16::MAX).unwrap();

        if (15..20).contains(&gap) {
            ones.write_sample(i16::MAX).unwrap();
        } else {
            ones.write_sample(0).unwrap();
        }

        if (35..39).contains(&gap) {
            zero.write_sample(i16::MAX).unwrap();
        } else {
            zero.write_sample(0).unwrap();
        }

        if (41..46).contains(&gap) {
            start.write_sample(i16::MAX).unwrap();
        } else {
            start.write_sample(0).unwrap();
        }
    }

    debug.finalize().unwrap();
    ones.finalize().unwrap();
    zero.finalize().unwrap();
    start.finalize().unwrap();

    for (i, section) in new_sections.iter().enumerate() {
        fs::write(format!("out/section-{i}.bin"), section.as_raw_slice()).unwrap();
    }
}

fn byte_eq(a: &BitSlice<u8>, b: &[bool]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for (a, b) in a.iter().zip(b.iter()) {
        if a != b {
            return false;
        }
    }

    true
}
