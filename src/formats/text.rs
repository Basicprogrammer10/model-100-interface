use std::{
    borrow::Cow,
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{bail, ensure, Context, Result};

use crate::{
    args::Decode,
    cassette::{self, Spec},
    formats::{checksum, FileType, Header},
    parser::BinParser,
};

pub fn decode(samples: &[i32], spec: Spec, args: Decode) -> Result<()> {
    let bin = cassette::decode(samples, spec)?;
    println!(" └─ Found {} sections", bin.len());

    println!("[*] Parsing file");
    let header = Header::parse(bin[0].as_raw_slice())?;
    ensure!(
        header.file_type == FileType::Text,
        "File is not a text file"
    );
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
        out.write_all(&section.data)?;
    }

    Ok(())
}

struct TextSection<'a> {
    data: Cow<'a, [u8]>,
    checksum: u8,
}

impl<'a> TextSection<'a> {
    /// Length of data should be exactly 0x100 bytes, right padded with 0x1A if necessary
    fn new(data: Cow<'a, [u8]>) -> Self {
        let mut checksum = 0_u8;
        for &byte in data.iter() {
            checksum = checksum.wrapping_add(byte);
        }

        Self {
            data,
            checksum: 0xFF - checksum,
        }
    }

    /// Create as many sections as necessary to fit the data
    fn new_multiple(data: Cow<'a, [u8]>) -> Vec<Self> {
        let mut sections = Vec::new();

        for chunk in data.chunks(0x100) {
            let mut chunk = chunk.to_vec();
            chunk.resize(0x100, 0x1A);
            sections.push(Self::new(Cow::Owned(chunk)));
        }

        sections
    }

    /// Encode the section into the format used on the cassette
    fn encode(&self) -> [u8; 278] {
        let mut out = [0; 278];

        out[0x00] = 0x8D;
        out[0x01..=0x100].copy_from_slice(&self.data);
        out[0x101] = self.checksum;

        out
    }

    /// Decode the section from the format used on the cassette
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
        let data = Cow::Borrowed(&bin[0x1..=end_pos]);
        let checksum = checksum(&bin[0x01..=0x101]);

        Ok(TextSection { data, checksum })
    }
}

#[cfg(test)]
mod test {

    use crate::{cassette::encode, formats::name};

    #[test]
    fn test_encode_segment() {
        use super::*;

        let header = Header::new(FileType::Text, name(b"TEST"), [0; 10]).encode();
        let raw_data = include_bytes!("../../README.md");
        let inner_data = TextSection::new_multiple(Cow::Borrowed(raw_data))
            .into_iter()
            .map(|e| e.encode())
            .collect::<Vec<_>>();

        let mut data = Vec::new();
        data.push(header.as_slice());
        for i in 0..inner_data.len() {
            data.push(inner_data[i].as_slice());
        }

        let spec = Spec {
            sample_rate: 44100,
            channels: 1,
            bits_per_sample: 16,
        };

        let encoded = encode(data.as_slice(), &spec).unwrap();

        let mut wav = hound::WavWriter::create(
            "output-test.wav",
            hound::WavSpec {
                channels: 1,
                sample_rate: 44100,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            },
        )
        .unwrap();

        for sample in encoded {
            wav.write_sample(sample).unwrap();
        }

        wav.finalize().unwrap();
    }
}
