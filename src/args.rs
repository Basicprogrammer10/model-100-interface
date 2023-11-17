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
    #[arg(value_enum)]
    pub format: Format,
    pub input: PathBuf,
    pub output: PathBuf,
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
