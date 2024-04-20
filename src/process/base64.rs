use crate::opt::Base64Format;
use anyhow::Result;
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine as _,
};
use std::fs::File;
use std::io::{stdin, Read};

pub fn process_encode(input: &str, format: Base64Format) -> Result<()> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encoded = match format {
        Base64Format::Standard => URL_SAFE_NO_PAD.encode(&buf),
        Base64Format::UrlSafe => STANDARD.encode(&buf),
    };

    println!("{}", encoded);
    Ok(())
}

pub fn process_decode(input: &str, format: Base64Format) -> Result<()> {
    let mut reader = get_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    let buf = buf.trim();

    let decoded = match format {
        Base64Format::Standard => URL_SAFE_NO_PAD.decode(buf),
        Base64Format::UrlSafe => STANDARD.decode(buf),
    };

    match decoded {
        Ok(decoded) => {
            let decoded = String::from_utf8(decoded)?;
            println!("{}", decoded);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    Ok(())
}

fn get_reader(input: &str) -> Result<Box<dyn Read>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_encode() {
        let input = "Cargo.toml";
        let format = Base64Format::Standard;
        process_encode(input, format).unwrap();
    }

    #[test]
    fn test_process_decode() {
        let input = "fixtures/b64.txt";
        let format = Base64Format::Standard;
        process_decode(input, format).unwrap();
    }
}
