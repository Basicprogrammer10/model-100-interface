//  Commands:
// - decode <type> <input> <output>
//  - type: raw, text

use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Decode(Decode),
}

#[derive(Parser)]
pub struct Decode {
    /// The format of the file to decode.
    #[arg(value_enum)]
    pub format: Format,
    /// The file or audio device to decode from.
    pub input: String,
    /// The file to write the output to.
    pub output: PathBuf,
    /// Weather the audio device should be used as input.
    #[arg(short, long)]
    pub audio_input: bool,
    /// Weather checksums should be ignored.
    /// Useful if the file is corrupted, but you still want to try to decode it.
    #[arg(short, long)]
    pub ignore_checksums: bool,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum Format {
    Raw,
    Text,
}

impl Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Raw => "raw",
            Format::Text => "text",
        })
    }
}
