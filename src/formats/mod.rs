use anyhow::{ensure, Result};

use crate::parser::BinParser;

pub mod raw;
pub mod text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileType {
    Text = 0x9C,
    Compiled = 0xD0,
    Basic = 0xD3,
}

struct Header {
    file_type: FileType,
    name: [u8; 6],
    misc: [u8; 10],
    checksum: u8,
}

impl FileType {
    fn from_u8(byte: u8) -> Result<Self> {
        match byte {
            0x9C => Ok(Self::Text),
            0xD0 => Ok(Self::Compiled),
            0xD3 => Ok(Self::Basic),
            _ => Err(anyhow::anyhow!("Invalid file type")),
        }
    }
}

impl Header {
    fn new(file_type: FileType, name: [u8; 6], misc: [u8; 10]) -> Self {
        let mut checksum = 0_u8;
        for &byte in name.iter().chain(misc.iter()) {
            checksum = checksum.wrapping_add(byte);
        }

        Self {
            file_type,
            name,
            misc,
            checksum: 0xFF - checksum,
        }
    }

    fn encode(&self) -> [u8; 0x26] {
        let mut out = [0; 0x26];

        out[0x00] = self.file_type as u8;
        out[0x01..=0x06].copy_from_slice(&self.name);
        out[0x07..=0x10].copy_from_slice(&self.misc);
        out[0x11] = self.checksum;

        out
    }

    fn parse(data: &[u8]) -> Result<Self> {
        ensure!(data.len() == 38, "Invalid header length");
        let mut parser = BinParser::new(data);

        let file_type = FileType::from_u8(parser.read_u8())?;
        let name = parser.read_array::<6>();
        let misc = parser.read_array::<10>();

        let checksum = checksum(&data[0x01..=0x11]);

        Ok(Header {
            file_type,
            name,
            misc,
            checksum,
        })
    }

    fn name(&self) -> String {
        self.name
            .iter()
            .take_while(|&&c| c != 0x20)
            .map(|&c| c as char)
            .collect()
    }
}

fn name(name: &[u8]) -> [u8; 6] {
    let mut out = [20; 6];

    let end = name.len().min(6);
    out[..end].copy_from_slice(&name[..end]);

    out
}

fn checksum(data: &[u8]) -> u8 {
    data.iter().fold(0_u8, |acc, &x| acc.wrapping_add(x))
}
