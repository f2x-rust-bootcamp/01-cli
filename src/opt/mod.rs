mod base64;
mod csv;
mod gen_pass;

use self::{csv::CsvOpts, gen_pass::GenPassOpts};
use clap::Parser;
use std::path::Path;

pub use base64::{Base64Format, Base64SubCommand};
pub use csv::OutputFormat;

#[derive(Debug, Parser)]
#[command(name = "mycli", version, about, author, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Convert CSV to other formats")]
    Csv(CsvOpts),

    #[command(name = "genpass", about = "Generate a password")]
    GenPass(GenPassOpts),

    #[command(subcommand)]
    Base64(Base64SubCommand),
}

pub fn verify_input_file(filename: &str) -> Result<String, &'static str> {
    if filename == "-" || Path::new(filename).exists() {
        Ok(filename.to_string())
        // Ok(filename.into())
    } else {
        Err("File does not exist")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_input_file("-"), Ok("-".into()));
        assert_eq!(verify_input_file("*"), Err("File does not exist"));
        assert_eq!(verify_input_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_input_file("nonexistent"), Err("File does not exist"));
    }
}
