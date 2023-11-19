use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{bail, ensure, Context, Result};

use crate::{
    args::Decode,
    cassette::{self, Spec},
    parser::BinParser,
};

pub fn decode(samples: &[i32], spec: Spec, args: Decode) -> Result<()> {
    let bin = cassette::decode(samples, spec)?;
    println!(" └─ Found {} sections", bin.len());

    println!("[*] Parsing file");
    let header = TextHeader::parse(bin[0].as_raw_slice())?;
    if header.checksum != 0x00 && !args.ignore_checksums {
        bail!("Invalid header checksum");
    }

    println!(" └─ File name: {}", header.name());

    let mut sections = Vec::new();
    for (i, e) in bin.iter().enumerate().skip(1) {
        let section =
            TextSection::parse(e.as_raw_slice()).with_context(|| format!("Section {i}"))?;
        if section.checksum != 0x00 && !args.ignore_checksums {
            bail!("Section {i} has an invalid checksum");
        }
        sections.push(section);
    }

    let out = File::create(args.output)?;
    let mut out = BufWriter::new(out);
    for section in sections {
        out.write_all(section.data)?;
    }

    Ok(())
}

// Note to anyone reading this:
// I have no idea how this format works, I am just making some guesses based on what I see in my hex editor.

struct TextHeader {
    name: [char; 6],
    name_len: u8,
    checksum: u8,
}

struct TextSection<'a> {
    data: &'a [u8],
    checksum: u8,
}

impl TextHeader {
    fn parse(bin: &[u8]) -> Result<Self> {
        ensure!(bin.len() == 38, "Invalid header length");
        let mut parser = BinParser::new(bin);
        ensure!(parser.read_u8() == 0x9C, "Non-text file");

        let name = [0; 6].map(|_| parser.read_u8() as char);
        let name_len = name.iter().position(|&c| c == ' ').unwrap_or(6) as u8;

        let checksum = checksum(&bin[0x01..=0x11]);

        Ok(TextHeader {
            name,
            name_len,
            checksum,
        })
    }

    fn name(&self) -> String {
        self.name.iter().take(self.name_len as usize).collect()
    }
}

impl<'a> TextSection<'a> {
    fn parse(bin: &'a [u8]) -> Result<Self> {
        ensure!(bin.len() == 278, "Invalid section length");
        let mut parser = BinParser::new(bin);
        ensure!(parser.read_u8() == 0x8D, "Missing start byte");

        let end_pos = bin
            .iter()
            .skip(0x1)
            .take(0x100)
            .rposition(|&x| x != 0x1A)
            .unwrap_or(0)
            + 0x1;

        ensure!(end_pos > 0, "Invalid end position");
        let data = &bin[0x1..=end_pos];
        let checksum = checksum(&bin[0x01..=0x101]);

        Ok(TextSection { data, checksum })
    }
}

fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0_u8, |acc, &x| acc.wrapping_add(x))
}
