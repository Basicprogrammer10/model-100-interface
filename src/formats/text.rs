use std::{
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{ensure, Context, Result};

use crate::{
    args::Decode,
    cassette::{self, Spec},
    parser::BinParser,
};

pub fn decode(samples: &[i32], spec: Spec, args: Decode) -> Result<()> {
    let bin = cassette::decode(samples, spec)?;
    println!(" └─ Found {} sections", bin.len());

    println!("[*] Parsing file");
    let header = TextHeader::parse(&bin[0].as_raw_slice())?;
    println!(" └─ File name: {}", header.name());

    let mut sections = Vec::new();
    for i in 1..bin.len() {
        let section =
            TextSection::parse(&bin[i].as_raw_slice()).with_context(|| format!("Section {i}"))?;
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
}

struct TextSection<'a> {
    data: &'a [u8],
}

impl TextHeader {
    fn parse(bin: &[u8]) -> Result<Self> {
        ensure!(bin.len() == 38, "Invalid header length");
        let mut parser = BinParser::new(bin);
        ensure!(parser.read_u8() == 0x9C, "Non-text file");

        let name = [0; 6].map(|_| parser.read_u8() as char);
        let name_len = name.iter().position(|&c| c == ' ').unwrap_or(6) as u8;

        Ok(TextHeader { name, name_len })
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
            .take(0xFF)
            .rposition(|&x| x != 0x1A)
            .unwrap_or(0xFF);
        ensure!(end_pos > 0, "Invalid end position");
        let end = bin[end_pos + 3] != 0xD1;
        let data = &bin[0x1..=end_pos + if end { 0 } else { 2 }];

        Ok(TextSection { data })
    }
}
