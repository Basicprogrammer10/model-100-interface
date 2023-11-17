use std::{
    fs::File,
    io::{BufReader, Write},
};

use anyhow::{Context, Result};
use hound::WavReader;

use crate::{args::Decode, cassette};

pub fn decode(reader: &mut WavReader<BufReader<File>>, args: Decode) -> Result<()> {
    let bin = cassette::decode(reader)?;
    println!("[I] {} sections", bin.len());

    let out_prefix = args.output.file_name().unwrap().to_string_lossy();

    for i in 0..bin.len() {
        let mut out = File::create(format!("{}-{}.bin", out_prefix, i)).unwrap();
        out.write_all(bin[i].as_raw_slice())
            .context("Writing to bin file")?;
    }

    Ok(())
}
