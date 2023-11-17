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

    let mut reader = hound::WavReader::open(&args.input)?;
    println!(
        "[I] {} channel{}, {} Hz, {} bit{}",
        reader.spec().channels,
        plural(reader.spec().channels),
        reader.spec().sample_rate,
        reader.spec().bits_per_sample,
        plural(reader.spec().bits_per_sample)
    );

    match args.format {
        args::Format::Raw => raw::decode(&mut reader, args)?,
        args::Format::Text => unimplemented!(),
    }

    println!("[*] Done!");
    Ok(())
}
