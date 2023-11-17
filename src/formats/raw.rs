use std::{fs::File, io::Write};

use anyhow::{Context, Result};

use crate::{
    args::Decode,
    cassette::{self, Spec},
};

pub fn decode(samples: &[i32], spec: Spec, args: Decode) -> Result<()> {
    let bin = cassette::decode(samples, spec)?;
    println!(" └─ Found {} sections", bin.len());

    let old_name = args.output.file_name().unwrap().to_string_lossy();
    let (prefix, ext) = old_name.rsplit_once('.').unwrap_or((old_name.as_ref(), ""));

    for i in 0..bin.len() {
        let mut out = File::create(format!("{prefix}-{i}.{ext}")).unwrap();
        out.write_all(bin[i].as_raw_slice())
            .context("Writing to bin file")?;
    }

    Ok(())
}
