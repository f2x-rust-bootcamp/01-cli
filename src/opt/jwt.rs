use super::verify_file;
use clap::Parser;

#[derive(Debug, Parser)]
pub enum JwtSubCommand {
    #[command(about = "Sign a string")]
    Sign(JwtSignOpts),

    #[command(about = "Verify a signed string")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(long)]
    pub sub: String,

    #[arg(long)]
    pub aud: String,

    #[arg(long)]
    pub exp: String,

    #[arg(short, long)]
    pub key: String,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub input: String,

    #[arg(short, long)]
    pub key: String,
}
