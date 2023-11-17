use anyhow::Result;

use crate::{
    args::{self, Decode},
    misc::plural,
};

mod raw;
mod text;

pub fn decode(args: Decode) -> Result<()> {
    println!(
        "[*] Decoding `{}` to `{}` ({})",
        args.input.to_string_lossy(),
        args.output.to_string_lossy(),
        args.format
    );

    let reader = hound::WavReader::open(&args.input)?;
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
