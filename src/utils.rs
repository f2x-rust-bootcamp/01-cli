use anyhow::Result;
use std::fs::File;
use std::io::{stdin, Read};

pub fn get_reader(input: &str) -> Result<Box<dyn Read>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}
