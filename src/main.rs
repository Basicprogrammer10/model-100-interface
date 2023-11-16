use std::fs::{self, File};

use bitvec::{
    order::{Lsb0, Msb0},
    slice::BitSlice,
    vec::BitVec,
};
use hound::{self, WavReader, WavSpec, WavWriter};

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

    {
        let spec = WavSpec {
            channels: 1,
            bits_per_sample: 16,
            ..spec
        };
        let mut debug = WavWriter::new(File::create("debug.wav").unwrap(), spec).unwrap();

        let mut last = 0;
        for i in intersections {
            let gap = i - last - 1;
            for _ in 0..gap {
                debug.write_sample(0).unwrap();
            }
            last = i;

            if (15..20).contains(&gap) {
                debug.write_sample(i16::MAX).unwrap();
            } else if (35..39).contains(&gap) {
                debug.write_sample(i16::MIN).unwrap();
            } else if (41..46).contains(&gap) {
                debug.write_sample(i16::MAX / -2).unwrap();
            } else if gap > 30000 {
                debug.write_sample(i16::MAX / 2).unwrap();
            } else {
                panic!("Invalid pulse length: {}", gap);
            }
        }

        debug.finalize().unwrap();
    }

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
            if !active
                && dat.len() >= 8
                && byte_eq(
                    &dat[dat.len() - 8..],
                    &[false, true, true, true, true, true, true, true],
                )
            {
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

fn byte_eq(a: &BitSlice<u8, Msb0>, b: &[bool]) -> bool {
    if a.len() != b.len() {
        panic!("Length mismatch");
        return false;
    }

    for (a, b) in a.iter().zip(b.iter()) {
        if a != b {
            return false;
        }
    }

    true
}
