use anyhow::Result;

use crate::{
    args::{self, Decode},
    misc::plural,
};

mod raw;

pub fn decode(args: Decode) -> Result<()> {
    println!(
        "[*] Decoding `{}` to `{}` ({})",
        args.input.to_string_lossy(),
        args.output.to_string_lossy(),
        args.format
    );

    let reader = hound::WavReader::open(&args.input)?;
    println!(
        "[I] {} channel{}, {} Hz, {} bit{}",
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
    println!("[I] {} samples", samples.len());

    match args.format {
        args::Format::Raw => raw::decode(&samples, spec, args)?,
        args::Format::Text => unimplemented!(),
    }

    println!("[*] Done!");
    Ok(())
}
